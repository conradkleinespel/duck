use crate::aes;
use crate::ffi;
use crate::password::PasswordError;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use rand::{rngs::OsRng, RngCore};
use rutil::rutil::safe_string::SafeString;
use rutil::rutil::safe_vec::SafeVec;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Error;
use std::fs::File;
use std::io::{
    Cursor, Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Seek, SeekFrom,
    Write,
};
use std::ops::Deref;
use std::os::raw::{c_uchar, c_ulonglong};

extern "C" {
    pub fn crypto_pwhash_scryptsalsa208sha256_ll(
        passwd: *const u8,
        passwdlen: usize,
        salt: *const u8,
        saltlen: usize,
        n: u64,
        r: u32,
        p: u32,
        buf: *mut u8,
        buflen: usize,
    ) -> libc::c_int;

    pub fn crypto_auth_hmacsha512(
        out: *mut libc::c_uchar,
        in_: *const libc::c_uchar,
        inlen: libc::c_ulonglong,
        k: *const libc::c_uchar,
    ) -> libc::c_int;

    pub fn crypto_auth_hmacsha512_verify(
        h: *const libc::c_uchar,
        in_: *const libc::c_uchar,
        inlen: libc::c_ulonglong,
        k: *const libc::c_uchar,
    ) -> libc::c_int;
}

/// The schema of the JSON content in the password file.
///
/// ```json
/// {
///     "passwords": [
///         "name": "YouTube",
///         "username": "conradk",
///         "password": "xxxxxxxx",
///         "created_at": 23145436,
///         "updated_at": 23145546,
///     ]
/// }
/// ```

/// The IV is 128 bits long.
///
/// This should be enough for it to be unique. Also, the password storage file
/// is normally unique, so if an attacker gets it, having access to the IV
/// doesn't help much, since there is no other data that uses the same IV to
/// compare with.
const IV_LEN: usize = 16;

/// Length of the key derived ffrom the user password, in bytes
const KEY_LEN: usize = 32;

/// Length of the salt passed to the key derivation function, in bytes
const SALT_LEN: usize = 32;

/// Length of the HMAC signature
const SIGNATURE_LEN: usize = 64;

/// Scrypt parameters
/// TODO: increase parameter strength by putting the parameters inside the Rooster file
const SCRYPT_PARAM_LOG2_N: u8 = 12;
const SCRYPT_PARAM_R: u32 = 8;
const SCRYPT_PARAM_P: u32 = 1;

/// The version of this lib
const VERSION: u32 = 2;

// Create a random IV.
fn generate_random_iv() -> IoResult<[u8; IV_LEN]> {
    let mut bytes: [u8; IV_LEN] = [0; IV_LEN];
    let mut rng = OsRng::default();
    rng.fill_bytes(&mut bytes);
    Ok(bytes)
}

// Create a random salt.
fn generate_random_salt() -> IoResult<[u8; SALT_LEN]> {
    let mut bytes: [u8; SALT_LEN] = [0; SALT_LEN];
    let mut rng = OsRng::default();
    rng.fill_bytes(&mut bytes);
    Ok(bytes)
}

/// Derives a 256 bits encryption key from the password.
fn generate_encryption_key(
    master_password: &str,
    salt: [u8; SALT_LEN],
    scrypt_log2_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
) -> SafeVec {
    let mut vec = Vec::<u8>::with_capacity(KEY_LEN);
    for _ in 0..KEY_LEN {
        vec.push(0u8);
    }
    let mut output = SafeVec::new(vec);

    let result = unsafe {
        // We use the low-level API because previous code was based off the unmaintained
        // crate rust-crypto, which used custom N, R and P parameters. These parameters are
        // only available in the low level libsodium API.
        crypto_pwhash_scryptsalsa208sha256_ll(
            master_password.as_ptr(),
            master_password.len(),
            salt.as_ptr(),
            salt.len(),
            1 << scrypt_log2_n as u64,
            scrypt_r,
            scrypt_p,
            output.as_mut_ptr(),
            KEY_LEN,
        )
    };

    if result != 0 {
        panic!("Deriving scrypt key failed: {:?}", result);
    }

    unsafe {
        output.inner.set_len(KEY_LEN);
    }

    output
}

/// Creates a HMAC signature
fn digest(
    key: &[u8],
    version: u32,
    scrypt_log2_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
    iv: &[u8],
    salt: &[u8],
    blob: &[u8],
) -> Result<Vec<u8>, PasswordError> {
    let blob_with_metadata =
        digest_blob_with_metadata(version, scrypt_log2_n, scrypt_r, scrypt_p, iv, salt, blob)?;

    let mut digest: Vec<u8> = Vec::with_capacity(512 / 8);

    let result = unsafe {
        crypto_auth_hmacsha512(
            digest.as_mut_ptr() as *mut c_uchar,
            blob_with_metadata.as_ptr() as *const c_uchar,
            blob_with_metadata.len() as c_ulonglong,
            key.as_ptr() as *const c_uchar,
        )
    };

    if result != 0 {
        panic!("Creating HMAC-SHA512 signature failed: {:?}", result);
    }

    unsafe {
        digest.set_len(512 / 8);
    }

    Ok(digest)
}

/// Creates the data that is signed with HMAC
fn digest_blob_with_metadata(
    version: u32,
    scrypt_log2_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
    iv: &[u8],
    salt: &[u8],
    blob: &[u8],
) -> Result<Vec<u8>, PasswordError> {
    let mut version_bytes_cursor: Vec<u8> = Vec::new();
    version_bytes_cursor.write_u32::<BigEndian>(version)?;
    let mut scrypt_bytes_cursor: Vec<u8> = Vec::new();
    scrypt_bytes_cursor.write_u8(scrypt_log2_n)?;
    scrypt_bytes_cursor.write_u32::<BigEndian>(scrypt_r)?;
    scrypt_bytes_cursor.write_u32::<BigEndian>(scrypt_p)?;
    let mut blob_with_metadata: Vec<u8> = Vec::new();
    blob_with_metadata.write_all(version_bytes_cursor.deref())?;
    blob_with_metadata.write_all(scrypt_bytes_cursor.deref())?;
    blob_with_metadata.write_all(iv)?;
    blob_with_metadata.write_all(salt)?;
    blob_with_metadata.write_all(blob.deref())?;
    Ok(blob_with_metadata)
}

/// The format of the encrypted JSON content in the password file v1.
#[derive(Serialize, Deserialize, Clone)]
pub struct Schema {
    passwords: Vec<Password>,
}

impl Schema {
    fn new() -> Schema {
        Schema {
            passwords: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Password {
    pub name: String,
    pub username: String,
    pub password: SafeString,
    pub created_at: ffi::time_t,
    pub updated_at: ffi::time_t,
}

impl Password {
    pub fn new<IS1: Into<String>, IS2: Into<String>, ISS: Into<SafeString>>(
        name: IS1,
        username: IS2,
        password: ISS,
    ) -> Password {
        let timestamp = ffi::time();
        Password {
            name: name.into(),
            username: username.into(),
            password: password.into(),
            created_at: timestamp,
            updated_at: timestamp,
        }
    }
}

pub struct PasswordStore {
    key: SafeVec,
    scrypt_log2_n: u8,
    scrypt_r: u32,
    scrypt_p: u32,
    salt: [u8; SALT_LEN],
    schema: Schema,
    master_password: String,
}

/// Read and writes to a Rooster file
///
/// The Rooster file has the following format:
/// - rooster version: u32, big endian
/// - scrypt log2n:    u8
/// - scrypt r:        u32, big endian
/// - scrypt p:        u32, big endian
/// - salt:            256 bits
/// - iv:              256 bits
/// - signature:       512 bits HMAC-SHA512
/// - encrypted blob:  variable length
impl PasswordStore {
    pub fn new(master_password: SafeString) -> IoResult<PasswordStore> {
        let salt = generate_random_salt()?;
        let key = generate_encryption_key(
            master_password.deref(),
            salt,
            SCRYPT_PARAM_LOG2_N,
            SCRYPT_PARAM_R,
            SCRYPT_PARAM_P,
        );

        Ok(PasswordStore {
            key: key,
            scrypt_log2_n: SCRYPT_PARAM_LOG2_N,
            scrypt_r: SCRYPT_PARAM_R,
            scrypt_p: SCRYPT_PARAM_P,
            salt: salt,
            schema: Schema::new(),
            master_password: master_password.into_inner(),
        })
    }

    pub fn from_input(
        master_password: SafeString,
        input: SafeVec,
    ) -> Result<PasswordStore, PasswordError> {
        let mut reader = Cursor::new(input.deref());

        // Version taken from network byte order (big endian).
        let version = reader.read_u32::<BigEndian>()?;
        if version != VERSION {
            if version > VERSION {
                return Err(PasswordError::OutdatedRoosterBinaryError);
            } else if version < VERSION {
                return Err(PasswordError::NeedUpgradeErrorFromV1);
            }
        }

        // Read the scrypt params.
        let scrypt_log2_n = reader.read_u8()?;
        let scrypt_r = reader.read_u32::<BigEndian>()?;
        let scrypt_p = reader.read_u32::<BigEndian>()?;

        // Read the old salt.
        let mut salt: [u8; SALT_LEN] = [0u8; SALT_LEN];
        reader.read(&mut salt).and_then(|num_bytes| {
            if num_bytes == SALT_LEN {
                Ok(())
            } else {
                Err(IoError::new(IoErrorKind::Other, "unexpected eof"))
            }
        })?;

        // Read the old IV.
        let mut iv: [u8; IV_LEN] = [0u8; IV_LEN];
        reader.read(&mut iv).and_then(|num_bytes| {
            if num_bytes == IV_LEN {
                Ok(())
            } else {
                Err(IoError::new(IoErrorKind::Other, "unexpected eof"))
            }
        })?;

        // Read the HMAC signature.
        let mut old_signature_mac: [u8; SIGNATURE_LEN] = [0u8; SIGNATURE_LEN];
        reader.read(&mut old_signature_mac).and_then(|num_bytes| {
            if num_bytes == SIGNATURE_LEN {
                Ok(())
            } else {
                Err(IoError::new(IoErrorKind::Other, "unexpected eof"))
            }
        })?;

        // The encrypted password data.
        let mut blob: Vec<u8> = Vec::new();
        reader.read_to_end(&mut blob)?;

        // Derive a 256 bits encryption key from the password.
        let key = generate_encryption_key(
            master_password.deref(),
            salt,
            scrypt_log2_n,
            scrypt_r,
            scrypt_p,
        );

        // Decrypt the data.
        let passwords = match aes::decrypt(blob.deref(), key.as_ref(), iv.as_ref()) {
            Ok(decrypted) => {
                let encoded = SafeString::from_string(
                    String::from_utf8_lossy(decrypted.as_ref()).into_owned(),
                );
                let s: Result<Schema, Error> = serde_json::from_str(encoded.deref());
                match s {
                    Ok(json) => json.passwords,
                    Err(_) => {
                        return Err(PasswordError::InvalidJsonError);
                    }
                }
            }
            Err(_) => {
                return Err(PasswordError::DecryptionError);
            }
        };

        // Check the signature against what it should be.
        let blob = digest_blob_with_metadata(
            version,
            scrypt_log2_n,
            scrypt_r,
            scrypt_p,
            &iv,
            &salt,
            blob.deref(),
        )?;

        let verification = unsafe {
            crypto_auth_hmacsha512_verify(
                old_signature_mac.as_ptr() as *const c_uchar,
                blob.as_ptr() as *const c_uchar,
                blob.len() as c_ulonglong,
                key.as_ptr() as *const c_uchar,
            )
        };
        if verification != 0 {
            return Err(PasswordError::CorruptionError);
        }

        Ok(PasswordStore {
            key: key,
            scrypt_log2_n: scrypt_log2_n,
            scrypt_r: scrypt_r,
            scrypt_p: scrypt_p,
            salt: salt,
            schema: Schema {
                passwords: passwords,
            },
            master_password: master_password.deref().into(),
        })
    }

    pub fn sync(&self, file: &mut File) -> Result<(), PasswordError> {
        // This should never fail. The structs are all encodable.
        let json_schema = match serde_json::to_string(&self.schema) {
            Ok(json_schema) => json_schema,
            Err(_) => {
                return Err(PasswordError::InvalidJsonError);
            }
        };
        let json_schema = SafeString::from_string(json_schema);

        // Encrypt the data with a new salt and a new IV.
        let iv = generate_random_iv()?;
        let encrypted = match aes::encrypt(
            json_schema.deref().as_bytes(),
            self.key.as_ref(),
            iv.as_ref(),
        ) {
            Ok(val) => val,
            Err(_) => return Err(PasswordError::EncryptionError),
        };

        // Reset the file pointer.
        file.seek(SeekFrom::Start(0))
            .and_then(|_| file.set_len(0))?;

        // Write the file version.
        file.write_u32::<BigEndian>(VERSION)?;

        // Write the scrypt params.
        file.write_u8(self.scrypt_log2_n)?;
        file.write_u32::<BigEndian>(self.scrypt_r)?;
        file.write_u32::<BigEndian>(self.scrypt_p)?;

        // Write the key derivation salt.
        file.write_all(&self.salt)?;

        // Write the encryption IV.
        file.write_all(&iv)?;

        // Write the file signature.
        let signature = digest(
            self.key.deref(),
            VERSION,
            self.scrypt_log2_n,
            self.scrypt_r,
            self.scrypt_p,
            &iv,
            &self.salt,
            encrypted.as_ref(),
        )?;
        file.write_all(signature.deref())?;

        // Write the encrypted password data.
        file.write_all(&encrypted.as_ref())?;

        file.sync_all()?;
        Ok(())
    }

    pub fn get_all_passwords(&self) -> Vec<&Password> {
        let mut passwords: Vec<&Password> = self.schema.passwords.iter().collect();

        passwords.sort_by_key(|p| {
            return p.name.to_lowercase();
        });

        passwords
    }

    /// Adds a password to the file.
    pub fn add_password(&mut self, password: Password) -> Result<(), PasswordError> {
        if password.password.deref().len() == 0 {
            return Err(PasswordError::EmptyPasswordError);
        }
        if self.has_password(password.name.deref()) {
            return Err(PasswordError::AppExistsError);
        }
        self.schema.passwords.push(password);
        Ok(())
    }

    pub fn delete_password(&mut self, name: &str) -> Result<Password, PasswordError> {
        let p = self
            .get_password(name)
            .ok_or(PasswordError::NoSuchAppError)?;

        let mut i = 0;
        while i < self.schema.passwords.len() {
            if self.schema.passwords[i].name == p.name {
                return Ok(self.schema.passwords.remove(i));
            }
            i += 1;
        }
        unreachable!();
    }

    pub fn search_passwords(&self, name: &str) -> Vec<&Password> {
        // Fuzzy search password app names.
        let keys = self
            .schema
            .passwords
            .iter()
            .map(|p| p.name.to_lowercase())
            .collect::<Vec<String>>();

        let mut search_results = vec![];
        // Check if each app name can be matched against the search query.
        //
        // It's fine if there are some characters left out in the query. For instance, you can
        // search for the app "Facebook" with just "fcbk".
        for app_name in keys.iter().map(|s| s.as_str()) {
            let mut matches_query = true;
            let mut last_i = 0;
            for c in name.chars() {
                let c_lowercase = format!("{}", c).to_lowercase();
                match app_name[last_i..].find(c_lowercase.as_str()) {
                    // Query chars must be present in the app name in the right order.
                    Some(ic) => {
                        last_i += ic + 1;
                    }
                    // Query char is not present, no match.
                    None => {
                        matches_query = false;
                        break;
                    }
                }
            }

            if matches_query {
                search_results.push(app_name.to_owned());
            }
        }

        let mut passwords = vec![];
        for p in self.schema.passwords.iter() {
            if search_results.contains(&p.name.to_lowercase()) {
                passwords.push(p);
            }
        }

        passwords.sort_by_key(|p| {
            return p.name.to_lowercase();
        });

        passwords
    }

    pub fn get_password(&self, name: &str) -> Option<Password> {
        'passwords_loop: for p in &self.schema.passwords {
            // Since the app name must be the same, we need the same length.
            if p.name.len() != name.len() {
                continue 'passwords_loop;
            }

            // We're looking for the exact same app name, without regard to casing.
            let mut i: usize = 0;
            while i < p.name.len() {
                let c1 = p.name.chars().nth(i).map(|c| c.to_lowercase().nth(0));
                let c2 = name.chars().nth(i).map(|c| c.to_lowercase().nth(0));
                if c1 != c2 {
                    continue 'passwords_loop;
                }
                i += 1;
            }
            return Some(p.clone());
        }
        None
    }

    pub fn has_password(&self, name: &str) -> bool {
        self.get_password(name).is_some()
    }

    pub fn change_password(
        &mut self,
        app_name: &str,
        closure: &dyn Fn(Password) -> Password,
    ) -> Result<Password, PasswordError> {
        let old_password = self.delete_password(app_name.deref())?;
        let new_password = closure(old_password.clone());
        match self.add_password(new_password.clone()) {
            Ok(_) => Ok(new_password),
            Err(err) => {
                // Password was not added, we'll add the old one back
                self.add_password(old_password)?;
                Err(err)
            }
        }
    }

    pub fn change_master_password(&mut self, master_password: &str) {
        self.key = generate_encryption_key(
            master_password,
            self.salt,
            self.scrypt_log2_n,
            self.scrypt_r,
            self.scrypt_p,
        );
    }

    pub fn change_scrypt_params(&mut self, scrypt_log2_n: u8, scrypt_r: u32, scrypt_p: u32) {
        self.scrypt_log2_n = scrypt_log2_n;
        self.scrypt_r = scrypt_r;
        self.scrypt_p = scrypt_p;

        self.key = generate_encryption_key(
            self.master_password.deref(),
            self.salt,
            self.scrypt_log2_n,
            self.scrypt_r,
            self.scrypt_p,
        );
    }
}

#[cfg(test)]
mod test {
    use crate::password::v2::{
        generate_encryption_key, generate_random_iv, generate_random_salt, Password, PasswordStore,
        SCRYPT_PARAM_LOG2_N, SCRYPT_PARAM_P, SCRYPT_PARAM_R,
    };
    use crate::password::PasswordError;
    use rutil::rutil::safe_string::SafeString;

    #[test]
    fn test_generate_random_iv_has_right_length() {
        assert_eq!(generate_random_iv().unwrap().len(), 16);
    }

    #[test]
    fn test_generate_random_salt_has_right_length() {
        assert_eq!(generate_random_salt().unwrap().len(), 32);
    }

    #[test]
    fn test_generate_encryption_key_returns_256_bits_key() {
        assert_eq!(
            generate_encryption_key(
                "hello world",
                generate_random_salt().unwrap(),
                SCRYPT_PARAM_LOG2_N,
                SCRYPT_PARAM_R,
                SCRYPT_PARAM_P
            )
            .len(),
            32
        );
    }

    #[test]
    fn test_create_password_store() {
        let store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();
        assert_eq!(store.get_all_passwords().len(), 0);
    }

    #[test]
    fn test_add_password() {
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();

        assert!(store
            .add_password(Password::new("name", "username", "password"))
            .is_ok());

        // need a wrap around the immutable borrow so the borrow checker is happy
        {
            // only the 1 password is here
            let passwords = store.get_all_passwords();
            assert_eq!(passwords.len(), 1);

            // is had the right information
            let p = passwords[0];
            assert_eq!(p.name, "name");
            assert_eq!(p.username, "username");
            assert_eq!(p.password, "password".into());
            assert_eq!(p.updated_at, p.created_at);
        }

        // cant add two passwords with same app name
        match store.add_password(Password::new("name", "username", "password")) {
            Err(PasswordError::AppExistsError) => {}
            _ => panic!(),
        }

        // empty password => not allowed
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();
        assert!(store
            .add_password(Password::new("name", "username", ""))
            .is_err());
    }

    #[test]
    fn test_change_password() {
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();

        assert!(store
            .add_password(Password::new("name", "username", "password"))
            .is_ok());
        assert_eq!(
            store
                .change_password("name", &|p| {
                    // change app name and password, keep username
                    Password::new("newname", p.username, "newpassword")
                })
                .unwrap(),
            Password::new("newname", "username", "newpassword")
        );
        assert_eq!(store.get_all_passwords().len(), 1);
        assert_eq!(store.get_all_passwords()[0].name, "newname");
        assert_eq!(store.get_all_passwords()[0].username, "username");
        assert_eq!(store.get_all_passwords()[0].password, "newpassword".into());

        // case insensitive works too
        assert_eq!(
            store.change_password("newname", &|p| p).unwrap(),
            Password::new("newname", "username", "newpassword")
        );
        assert_eq!(store.get_all_passwords().len(), 1);
        assert_eq!(store.get_all_passwords()[0].name, "newname");
        assert_eq!(store.get_all_passwords()[0].username, "username");
        assert_eq!(store.get_all_passwords()[0].password, "newpassword".into());

        // empty password => do not change anything
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();
        assert!(store
            .add_password(Password::new("name", "username", "password"))
            .is_ok());
        assert!(store
            .change_password("name", &|p| {
                // change app name and password, keep username
                Password::new(p.username.clone(), p.username.clone(), "")
            })
            .is_err());
        assert_eq!(store.get_all_passwords()[0].name, "name");
        assert_eq!(store.get_all_passwords()[0].username, "username");
        assert_eq!(store.get_all_passwords()[0].password, "password".into());
    }

    #[test]
    fn test_delete_password() {
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();

        assert!(store
            .add_password(Password::new("name1", "username", "password"))
            .is_ok());
        assert!(store
            .add_password(Password::new("name2", "username", "password"))
            .is_ok());
        assert_eq!(store.get_all_passwords().len(), 2);

        assert_eq!(
            store.delete_password("name1").unwrap(),
            Password::new("name1", "username", "password")
        );
        assert!(store.get_password("name1").is_none());
        assert_eq!(store.get_all_passwords().len(), 1);
        // case insensitive works too
        assert_eq!(
            store.delete_password("NAME2").unwrap(),
            Password::new("name2", "username", "password")
        );
        assert!(store.get_password("name2").is_none());
        assert_eq!(store.get_all_passwords().len(), 0);
    }

    #[test]
    fn test_get_password() {
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();

        assert_eq!(store.get_password("name"), None);
        assert!(store
            .add_password(Password::new("name", "username", "password"))
            .is_ok());
        assert_eq!(
            store.get_password("name").unwrap(),
            Password::new("name", "username", "password")
        );
        assert_eq!(
            store.get_password("NaMe").unwrap(),
            Password::new("name", "username", "password")
        );
    }

    #[test]
    fn test_has_password() {
        let mut store = PasswordStore::new(SafeString::from_string("****".to_owned())).unwrap();

        assert!(!store.has_password("name"));
        assert!(store
            .add_password(Password::new("name", "username", "password"))
            .is_ok());
        assert!(store.has_password("name"));
    }
}

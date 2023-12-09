# Duck, a monorepo for my open source projects

This repository contains most of the code for my open source projects.

![](duck.png)

## Development environment

In order to maintain better security and isolation between workstation and duck during development, you can use
a virtual machine:

- Download Manjaro Linux XFCE edition: https://manjaro.org/download/;
- Create a new VirtualBox VM with Manjaro, user is `duck`, password is `duck`, including for `root`;
- Configure VM to forward port 22 to host port 2222;
- Add a VirtualBox shared folder pointing duck to `/mnt/duck`;
- Run the following inside the VM:
  ```shell
    sudo /mnt/duck/setup-manjaro.sh
  ```
- Logout and log back in for changes to take effect;
- Run the following inside the VM:
  ```shell
    /mnt/duck/setup-user.sh
  ```
- Configure your IDE / code editor to execute code within the VM via SSH.
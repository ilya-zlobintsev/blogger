# Cross compiling rust projects with cross

If you've ever wanted to compile your Rust programs for a device with a different architecture such as a Raspberry Pi or a router running OpenWRT, this will help you.

## Explanation on what the problem is

`cargo` actually does come with built-in support for building for different target architectures! If you use rustup, you can view the list of supported architectures with `rustup target list`. It will give you a long list of all architectures supported by rust. 

You can install support for a given architecture and try building the project, but now you will be given a huge amount of scary-looking errors:
`cargo build --target=aarch64-unknown-linux-gnu`:

`          /usr/bin/ld: /tmp/tmp.c0k3H5Uuxt/testproject/target/aarch64-unknown-linux-gnu/debug/deps/asdf-a02e73b3d7342c9c.1lb2dql4hga2wj0s.rcgu.o: error adding symbols: file in wrong format
          collect2: error: ld returned 1 exit status`

This is a linking error: linking is the stage of compiling your program where it gets bundled into a binary for the given architecture. The error occurs because cargo is trying to use your system's linker (in this case a program called `ld`), which is for a different architecture and cannot be used to build binaries for the architecture you gave to cargo.

The manual way of compiling for a different architecture is to get a cross-compiling toolchain for the architecture you need. You can still do that, however: building the toolchain, making sure you have the right one and have properly configured cargo to use the needed linker for any given project can get very tedious and complicated.

### This is where cross comes in.

[Cross](https://github.com/rust-embedded/cross) is a utility that lets you build Rust projects for different architectures using containers. It does so by maintaining a [library of containers images](https://hub.docker.com/r/rustembedded/cross/tags?page=1&ordering=last_updated) pre-configured with cross-compilation toolchanis.

#### There is a prerequisite for using cross, either:
- Docker installed and running, with your user added to the `docker` group to be able to use it wihout `sudo`
- Podman that's configured for rootless container support (Arch wiki has a [good article](https://wiki.archlinux.org/index.php/Podman#Rootless_Podman) on how to set it up, applicable for distros other than Arch as well)

If you've never used containers before, there are plenty of beginner guides on how to set the environment up. But generally, all you need is to install docker and add yourself to the group.


#### After you have a container runtime set up, you need to install cross itself.

Arch Linux has cross in the official repositories:

`sudo pacman -S cross`

If your distro doesn't have it, you can use cargo instead:

`cargo install cross`

#### That's it!

Cross works as a wrapper for cargo and uses the same syntax, so it's pretty straightfoward to use.

Go to your project and try to build it, substituting cargo with cross. For example:

`cross build --target=aarch64-unknown-linux-gnu`

If everything goes well, your container engine will first pull a toolchain image for the architecture:

```
Trying to pull docker.io/rustembedded/cross:aarch64-unknown-linux-gnu-0.2.1...
Getting image source signatures
Copying blob 29047100b040 done  
Copying blob 4b6bc9e29877 done  
Copying blob b5e173e44934 done  
Copying blob 45653be35864 done  
Copying blob 190d82667f73 done  
Copying blob 15743a713c2a done  
Copying blob 1abcae387ca7 done  
Copying blob dddd453e4629 done  
Copying blob 0999ab463a86 done  
Copying blob 1316155995d7 done  
Copying blob e4b709d5b7af done  
Copying blob 0aebb58bbbd6 done  
Copying blob ca33887788d7 done  
Copying blob 01194c4c0f93 done  
Copying blob 27b9ee115194 done  
Copying blob ae083dc71cb1 done  
Copying blob 37a2a10584c8 done  
Copying blob b6bb12e7d793 done  
Copying config 244b436091 done  
Writing manifest to image destination
Storing signatures
```

And then it will start building your project as it normally does with cargo. The output will be stored in `target/{architecture_name}` (for example in `target/aarch64-unknown-linux-gnu/debug`). Now you can copy the binary to the device you intended to run the program on and it should just work.

#### Building projects that require SSL

If you try to build a project that requires something that uses SSL/TLS you will be met with a following error:

```
run pkg_config fail: "`\"pkg-config\" \"--libs\" \"--cflags\" \"openssl\"` did not exit successfully: exit code: 1
Package openssl was not found in the pkg-config search path.
Perhaps you should add the directory containing `openssl.pc' to the PKG_CONFIG_PATH environment variable
No package 'openssl' found"
```

This happens because it's trying to use OpenSSL, and thus needs OpenSSL's header files to compile. Since the toolchain container doesn't have them included, it doesn't work.

There are two solutions for this problem:
- Use rustls instead if OpenSSL:

	If your SSL-dependant library supports it, you can specify it to use rutls. Since it's in Rust, it gets compiled by cross for the target platform aswell. 

	Example with reqwest:

	`reqwest = { version = "", default-features = false, features = ["rustls-tls"] }`
	
	(you need to disable default-features in order for it to not use OpenSSL.)

- Make cross compile openssl-sys:

	If you can't use the rustls option, it's also possible to make cross compile OpenSSL by itself.

	Most programs/libraries don't use OpenSSL directly, but instead make use of the `native-tls` crate (the native TLS implementation on Linux happens to be OpenSSL). It's possible to specify the `vendored` feature in native-tls. Another example with reqwest:

	```
	reqwest = ""
    native-tls = { version = "", features = ["vendored"] }
    ```

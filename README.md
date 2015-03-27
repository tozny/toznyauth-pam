toznyauth-pam
=============

Authenticate to Linux-based systems using [Tozny][].

Tozny provides `toznyauth_pam` adding Tozny support to common Unix commands,
such as ssh and sudo.
`toznyauth_pam` is Based on the PAM framework,
a pluggable authentication framework that is supported on Linux, OS X, and several BSD flavors.
With `toznyauth_pam` you can add two-factor authentication to Unix services on a per-service basis.
Or you can use Tozny as a primary authentication factor.

`toznyauth_pam` is a beta feature.
Please use caution when trying it to avoid locking yourself out of a server in
case something goes wrong.

[Tozny]: http://tozny.com/

For background, and for details on installing and configuring, see the [full documentation][docs].

[docs]: http://tozny.com/documentation/integration/linux/


## Try it

A demo is included in the `demo/` directory.
The demo builds the PAM module in a Docker container,
and runs an ssh server that is configured to use Tozny authentication.

Before trying the demo, install the Tozny app for [Android][] or [iOS][].
Then try logging in to the [web-based demo][bank-demo] to get a Tozny user id.

[Android]: https://play.google.com/store/apps/details?id=com.tozny.authenticator
[iOS]: https://itunes.apple.com/us/app/tozny/id855365899?mt=8
[bank-demo]: https://demo.tozny.com/bank/index.php

To find your Tozny user id and realm id, open the Tozny app and select
"Accounts".
Select an identity (one should have been added when you logged into the web demo.)
You should see your user id and realm id.
They are both of the format 'sid_c233df00c07b9'.

Next run the PAM demo:

    $ demo/run.sh

Enter your Tozny user id and realm id.

The demo will download dependencies and run the build process in a Docker
container; so it will take a little while to start up.
Once it is running, you will have a local ssh server to log into.


## Building

This project is currently building with:

> rustc 1.0.0-nightly (b4c965ee8 2015-03-02) (built 2015-03-03)

To get that particular version of the rust compiler:

    $ curl -O https://static.rust-lang.org/dist/2015-03-03/rust-nightly-x86_64-unknown-linux-gnu.tar.gz
    $ tar -xzf rust-nightly-x86_64-unknown-linux-gnu.tar.gz
    $ cd rust-nightly-x86_64-unknown-linux-gnu
    $ sudo ./install.sh

Install development dependencies.  On Debian-based systems those are:

    $ sudo apt-get install build-essential libpam-dev

Build:

    $ cargo build

Because rust is still alpha, there is currently a lot of churn in the standard
libraries and in the dependencies of this crate.  To deal with that,
a Dockerfile is included which provides a reproducible build environment.  To
build with Docker (instead of using cargo directly) run:

    $ make

This will output `dist/toznyauth_pam.so`


## Testing

Install `pamtester`.

Create a test service configuration file, `/etc/pam.d/test-service` with this
content:

    $ echo 'auth required toznyauth_pam.so' | sudo tee /etc/pam.d/test-service

Test:

    $ pamtester test-service $USER "authenticate(0)"

Where `$USER` can be any Linux user on your system, and `0` can be replaced with
any valid PAM flags.

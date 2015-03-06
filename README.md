toznyauth-pam
=============

Authenticate to Linux-based systems using [Tozny][]. Based on the PAM framework.

This project is currently building with:

> rustc 1.0.0-nightly (b4c965ee8 2015-03-02) (built 2015-03-03)

Earlier or later versions of the rust compiler are unlikely to work.

[Tozny]: http://tozny.com/


## Building

Install rust and cargo:

    $ curl -s https://static.rust-lang.org/rustup.sh | sudo sh

Build:

    $ cargo build

Because rust is still alpha, there is currently a lot of churn in the standard
libraries and in the dependencies of this crate.  To deal with that,
a Dockerfile is included which provides a reproducible build environment.  To
build with Docker (instead of using cargo directly) run:

    $ make

This will output `dist/toznyauth_pam.so`


## Installing

The build process produces a file of the format:

    target/libtoznyauth_pam-[hex fingerprint].so

Copy that file to:

    /lib/security/toznyauth_pam.so


## System configuration

Pam configuration is typically found in `/etc/pam.d/`, where there is
a configuration file for each service that uses pam for authentication.  You
have the option to use Tozny authentication for just one particular service, or
for many services.  There is also a lot of flexibility for using Tozny as
a second factor as a primary authentication factor.

Create or edit a service configuration file in `/etc/pam.d/` that uses
toznyauth_pam.  A minimal configuration that uses toznyauth_pam as a second factor looks
like this:

    @include common-auth
    auth required toznyauth_pam.so

`common-auth` contains configuration that is shared by most services.  It
enables the usual username-and-password authentication.  Placing the toznyauth_pam
line after common-auth causes pam to prompt for standard authentication first,
and then to prompt for pam authentication.

A configuration that uses toznyauth_pam as a single factor, but that falls back to
standard authentication if Tozny authentication fails:

    auth sufficient toznyauth_pam.so
    @include common-auth

Finally, a configuration that uses toznyauth_pam exclusively:

    auth required toznyauth_pam.so

toznyauth_pam accepts command line arguments, which are given at the end of the line
in the config file.  For example:

    auth required toznyauth_pam.so --no-presence --no-mobile

This configures toznyauth_pam to disable push notifications, and to disable display
of the mobile `tozauth://` URL.

The available options are:

- `--prompt`, prompts the user to press Enter, which might be required with sshd
- `--no-qr`, disables inline QR code display (a URL for a QR code is still
  displayed)
- `--no-presence`, disables push notifications, which might be desirable if
  multiple people share one user account
- `--no-mobile`, disables display of the mobile `tozauth://` URL


### Tozny and sudo

Pam configuration for sudo is typically found in `/etc/pam.d/sudo`.  It might
look something like this:

    #%PAM-1.0
    
    @include common-auth
    @include common-account
    @include common-session-noninteractive

Simply put a toznyauth_pam line immediately above or below the `common-auth` line,
depending on whether you want to use Tozny as a second or as a primary factor:

    #%PAM-1.0
    
    @include common-auth
    auth required toznyauth_pam.so
    @include common-account
    @include common-session-noninteractive

It is important that all of the auth lines come before the `common-account` and
`common-session-noninteractive` lines.


### Tozny and OpenSSH

Using toznyauth_pam with sshd comes with some specific considerations.  OpenSSH uses
its own authentication system, and only calls into pam if its
`keyboard-interactive` authentication mode is enabled.

Configuration for sshd is typically found in `/etc/ssh/sshd_config`.  Make sure
your configuration includes these options:

    ChallengeResponseAuthentication yes
    UsePAM yes

If you want to restrict keyboard-interactive authentication or use of pam, you
can make use of [sshd's filtering options][Match].  For example:

    Match User deploy Address 172.16.1.*
        ChallengeResponseAuthentication yes

[Match]: https://raymii.org/s/tutorials/Limit_access_to_openssh_features_with_the_Match_keyword.html

By default sshd will try key-based authentication first, and will fall back to
keyboard-interactive if the user connecting does not have an authorized key.

Another caveat is that in at least some environments, sshd will not display
informational messages from a pam module.  It will only display prompts.  This
is why toznyauth_pam includes a `--prompt` option, which prompts the user to press
Enter after scanning a code or confirming a push notification.  If your ssh
connection appears to hang, and you see no output then you probably need to use
the `--prompt` option.

Pam configuration for sshd is typically found in `/etc/pam.d/sshd`.  The
simplest option is to add a line above the `common-auth` include:

    auth sufficient toznyauth_pam.so --prompt

    # Standard Un*x authentication.
    @include common-auth

Or for two-factor authentication:

    # Standard Un*x authentication.
    @include common-auth
    auth required toznyauth_pam.so --prompt


## Per-user configuration

toznyauth_pam requires a file in a user's home directory to specify which Tozny
users are authorized to log into the corresponding unix account.  The file
should be in `~/.config/tozny/authorized.toml`  The file must be in [TOML][]
format.  Here is a complete example:

    realm_key_id = "sid_e887ece438ff1"
    authorized_users = [ "sid_ef66781ebcdc0" ]

The `realm_key_id` field specifies the Tozny realm that toznyauth_pam should use for
authentication.  The `authorized_users` array contains zero or more Tozny user
ids, indicating which Tozny users are authorized.

An optional `api_url` field is also accepted.  In most cases this should be left
as the default value, `"https://api.tozny.com"`.

[TOML]: https://github.com/toml-lang/toml

A user id can be found by selecting "Accounts" in the Tozny mobile app.
Or in the Tozny admin interface, when viewing a user's profile you can get the
user id from the page URL.  The user and realm key ids are always in the format
of the string "sid\_" followed by 13 hexadecimal characters.


## Testing

Install `pamtester`.

Create a test service configuration file, `/etc/pam.d/test-service` with this
content:

    $ echo 'auth required toznyauth_pam.so' | sudo tee /etc/pam.d/test-service

Test:

    $ pamtester test-service $USER "authenticate(0)"

Where `$USER` can be any Linux user on your system, and `0` can be replaced with
any valid PAM flags.

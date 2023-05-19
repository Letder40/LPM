# LPM ( Letder's Password Manager )
LPM is a secure TUI password manager, with it you can save yours passwords in a encrypted file named passfile.lpm.

## Security
* passfile.lpm is encrypted in aes\_gcm256
* The input of the master key is stored as String type value but it is inmediatly hashed to sha256
* The variable that stored the input of the master key is seroized after hash it.

## Basic usage
![Help LPM](media/help.png)


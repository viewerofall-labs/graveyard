# Global aur/curl command install

This only includes the testing for .sh installation commands and aur packages via the pkgbuild. 

## Aur
> Dont question it <small>We use questionable methods</small>

This works by pulling the PKGBUILD, executing it via making a folder in ~ and treating it as home with new folders and such. It will create **A .log file for troubleshooting** if something goes wrong in order to find out the error and fix it. 
### Aur usage
```bash
./dead/testinstall/aur-test-harness.sh ~/PKGBUILD
```

## Curl
This requires a simple get.sh or install.sh installer for programs it would be preferred if you didnt have to clone the whole repo for that since I have not tested it and do not plan on it. You simply download the get.sh installer, run it through it and it puts it in a folder with the installed varibles and files from it. Simple as that

### Curl usage
```bash 
./dead/testinstall/woven-test-harness.sh ~/get.sh
```


A quick program to help with syncing your lethal company mods with a modpack of your choice on Github.

This was developed for personal use, I have a script to handle it on Linux easily but friends playing on Windows had no equivalent.
Instead of telling them to install Git and run a script etc. I just wrote this to be able to compile a .exe for them.

# HOW TO DOWNLOAD

Go into the releases tab and download the latest .exe file for Windows users.

If you're using linux you probably don't need this repo but if you request it I will add a Linux build + guide

# HOW TO RUN THE .EXE (in releases tab)

Move the .exe file in the Lethal Company folder (e.g `C:\\Program Files\Steam\steamapps\common\Lethal Company\`) at the same level as the lethal compny .exe file or the BepInEx folder

It should now run the Lethal Company.exe automatically after finishing the sync.

# HOW TO REPLACE LETHAL COMPANY EXE BYTHIS

Go on steam, Launch options and write the full path to the file (e.g `C:\\Program Files\Steam\steamapps\common\Lethal Company\`)

## YOU SHOULD HAVE BEPINEX INSTALLED AND SETUP BEFORE USING THIS

Double click on the .exe file, wait for the download and copy.
If everything goes as intended, you should have a terminal open with a progress bar, that closes automatically at the end of the files copy.

The program creates a `last_not_so_serious_company_commit.log` with the latest commit hash from the modpack you use.

If you need to rerun the .exe and your modpack hasn't had any change recently, remove the file before reruning the .exe

It only serves as a check to prevent the .exe from downloading the whole modpack again if nothing changed.

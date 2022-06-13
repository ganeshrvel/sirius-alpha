# Sirius Alpha | IOT Module for ESP32 based chips

- Author: [Ganesh Rathinavel](https://www.linkedin.com/in/ganeshrvel "Ganesh Rathinavel")
- License: [MIT](https://github.com/ganeshrvel/sirius-alpha/blob/master/LICENSE "MIT")
- Repo URL: [https://github.com/ganeshrvel/sirius-alpha](https://github.com/ganeshrvel/sirius-alpha/ "https://github.com/ganeshrvel/sirius-alpha")
- Contacts: ganeshrvel@outlook.com

# Introduction
Sirius Alpha is an IOT module which runs on the ESP32 chips, they are safe and super fast.


### Device setup
```shell
curl -fsSL https://raw.githubusercontent.com/platformio/platformio-core-installer/master/get-platformio.py -o get-platformio.py
python3 get-platformio.py

cargo install cargo-pio --git https://github.com/ivmarkov/cargo-pio
```


Rename .sample.env.yaml as .env.yaml and edit the values

```shell
#install node 16 or above
npm -g i nvm

#use node 16 or above
nvm use 16

#install zx globally
npm -g i zx
```

```shell
cd ./scripts
./build.sh
```

- `.env` file will get automatically generated after running the `./build.sh` file

### References
- TLS demo https://github.com/killyourphone/tlsdemo

- You will need to source the setup_envs.sh file before things will work! That sets up the correct compiler overrides for ring's cc based build script.
```shell
source ./scripts/setup_envs.sh
```


### Contacts
Please feel free to contact me at ganeshrvel@outlook.com

### License
Sirius Alpha | IOT Module for ESP32 based chips is released under [MIT License](https://github.com/ganeshrvel/sirius-alpha/blob/master/LICENSE "MIT License").

Copyright © 2018-Present Ganesh Rathinavel

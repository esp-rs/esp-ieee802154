set INCL=%HOMEPATH%\.espressif\tools\riscv32-esp-elf\esp-2021r2-8.4.0\riscv32-esp-elf\riscv32-esp-elf\include\
set OPTS=--no-derive-debug --raw-line "#![allow(non_camel_case_types,non_snake_case,non_upper_case_globals,dead_code,improper_ctypes)]" --use-core --ctypes-prefix "crate::binary::c_types" --no-layout-tests 
bindgen %OPTS% include\include.h > src\binary\include_esp32h4.rs -- -I./headers/ -I%INCL% -I./include/ -DCONFIG_IDF_TARGET_ESP32H4 -I./headers/esp32h4/

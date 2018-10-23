
If you're on a Mac and have the PCAN adaptor attached, you should run the
application with the `pcan` feature. For it to find the native library you
have to set the dynamic library loading path:

    export LD_LIBRARY_PATH=./lib/PCBUSB
    cargo run --features pcan tests/stubs/simple.json
    
If you're not on a Mac then you can run the unit tests, but there are no 
adaptors yet for CAN hardware.

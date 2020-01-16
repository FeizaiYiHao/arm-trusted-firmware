The following is the call sequence.

* The entry point is fsp_entrypoint, which is in asm/fsp_entrypoint.S.
* It calls fsp_c_main(), which is in fsp_c_main.c. This is in C.
* It calls fsp_main(), which is in fsp/src/lib.rs. This is in Rust.
* It calls fsp_print_debug_message(), which is in fsp_c_main.c. This is in C.

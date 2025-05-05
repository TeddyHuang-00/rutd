# Set temporary root path for demonstration purposes
export RUTD_PATH__ROOT_DIR := "/tmp/rutd"

# Clean up the temporary root directory from previous runs
_demo_clean:
    rm -rf $RUTD_PATH__ROOT_DIR

_demo_help:
    vhs assets/tapes/help.tape

_demo_clone_sync: _demo_clean
    vhs assets/tapes/clone-sync.tape

_demo_add: _demo_clone_sync
    vhs assets/tapes/add.tape

_demo_start_stop: _demo_add
    vhs assets/tapes/start-stop.tape

_demo_done_abort: _demo_start_stop
    vhs assets/tapes/done-abort.tape

_demo_filter: _demo_done_abort
    vhs assets/tapes/filter.tape

_demo_complete: _demo_done_abort
    vhs assets/tapes/advance-complete.tape

demo: _demo_help _demo_clone_sync _demo_add _demo_start_stop _demo_done_abort _demo_filter _demo_complete
    @echo "Demo completed successfully!"
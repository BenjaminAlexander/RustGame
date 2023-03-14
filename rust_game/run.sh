#!/bin/bash

gnome-terminal -e "target/debug/rust_game -s"
gnome-terminal -e "target/debug/rust_game -c Client1"
gnome-terminal -e "target/debug/rust_game -c Client2"
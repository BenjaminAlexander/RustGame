#!/bin/bash

gnome-terminal -e "target/debug/simple-game -s"
gnome-terminal -e "target/debug/simple-game -c Client1"
gnome-terminal -e "target/debug/simple-game -c Client2"

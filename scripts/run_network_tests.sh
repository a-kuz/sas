#!/bin/bash

cargo test --test network_basic --test network_movement --test network_animation -- --nocapture --test-threads=1


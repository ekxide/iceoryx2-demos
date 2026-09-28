#!/bin/bash

set -e

declare -A pids=()

build() {
    echo
    echo "build reference system ... this might take a while"
    cargo build -p automotive-reference-system --release 2>&1 | while IFS= read -r line; do
        printf '\r\033[K%s' "$line"
    done
    printf '\n'
}

launch() {
    echo
    echo "launching reference system"
    name="Intersection Output"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Euclidean Cluster Detector" "--interval-in-ms" "100" "actor" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Vehicle DBW System"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Vehicle Interface" "--interval-in-ms" "100" "actor" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Behavior Planner"
    echo "  $name"
    $(./target/release/automotive-reference-system "-c" "100" "-e" "-n" "$name" "-s" "Object Collision Estimator" "-s" "NDT Localizer" "-s" "Lanelet2Global Planner" "-s" "Lanelet2Map Loader" "-s" "Parking Planner" "-s" "Lane Planner" "--interval-in-ms" "250" "processing" "-o" "Behavior Planner" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Euclidean Cluster Detector"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Ray Ground Filter" "-s" "Euclidean Cluster Settings" "--interval-in-ms" "100" "processing" "-o" "Euclidean Cluster Detector" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Front Points Transformer"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Front Lidar Driver" "--interval-in-ms" "100" "processing" "-o" "Front Points Transformer" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Lane Planner"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Lanelet2Map Loader" "--interval-in-ms" "100" "processing" "-o" "Lane Planner" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Lanelet2Global Planner"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "NDT Localizer" "-s" "Visualizer" "--interval-in-ms" "100" "processing" "-o" "Lanelet2Global Planner" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Lanelet2Map Loader"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Lanelet2Global Planner" "-s" "Lanelet2Map" "--interval-in-ms" "100" "processing" "-o" "Lanelet2Map Loader" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="MPC Controller"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Behavior Planner" "--interval-in-ms" "100" "processing" "-o" "MPC Controller" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="NDT Localizer"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Voxel Grid Downsampler" "-s" "Point Cloud Map Loader" "--interval-in-ms" "100" "processing" "-o" "NDT Localizer" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Object Collision Estimator"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Euclidean Cluster Detector" "--interval-in-ms" "100" "processing" "-o" "Object Collision Estimator" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Parking Planner"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Lanelet2Map Loader" "--interval-in-ms" "100" "processing" "-o" "Parking Planner" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Point Cloud Fusion"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Front Points Transformer" "-s" "Rear Points Transformer" "--interval-in-ms" "100" "processing" "-o" "Point Cloud Fusion" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Point Cloud Map Loader"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Point Cloud Map" "--interval-in-ms" "100" "processing" "-o" "Point Cloud Map Loader" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Ray Ground Filter"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Point Cloud Fusion" "--interval-in-ms" "100" "processing" "-o" "Ray Ground Filter" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Rear Points Transformer"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Rear Lidar Driver" "--interval-in-ms" "100" "processing" "-o" "Rear Points Transformer" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Vehicle Interface"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "MPC Controller" "-s" "Behavior Planner" "--interval-in-ms" "100" "processing" "-o" "Vehicle Interface" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Voxel Grid Downsampler"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Point Cloud Fusion" "--interval-in-ms" "100" "processing" "-o" "Voxel Grid Downsampler" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Euclidean Cluster Settings"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Euclidean Cluster Settings" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Lidar"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Rear Lidar Driver" "-s" "Front Lidar Driver" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Point Cloud Map"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Point Cloud Map" "-s" "Lanelet2Map" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1) &
    pids[$!]="$name"

    name="Visualizer"
    echo "  $name"
    $(./target/release/automotive-reference-system "-e" "-n" "$name" "-s" "Visualizer" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1) &
    pids[$!]="$name"

}

check_processes() {
    declare -A remaining=()

    for pid in "${!pids[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            remaining[$pid]="${pids[$pid]}"
        else
            if wait "$pid" 2>/dev/null; then
                status=0
            else
                status=$?
            fi

            echo "'${pids[$pid]}' (PID $pid) is no longer running (exit status $status)"
        fi
    done

    pids=()
    for pid in "${!remaining[@]}"; do
        pids["$pid"]="${remaining[$pid]}"
    done
}

build
launch

while ((${#pids[@]} > 0)); do
    check_processes
    sleep 1
done

#!/bin/bash


sighandler() {
    echo "Caught SIGINT, terminating all child processes..."
    kill -- -$$ 2>/dev/null
    exit 1
}

trap sighandler SIGINT SIGTERM

verify_success() {
    failed=0
    for pid in "${pids[@]}"; do
        if ! wait $pid; then
            failed=1
        fi
    done
    
    if [ "$failed" -eq 1 ]; then
        echo "launch process group failed"
        exit -1
    fi
}


echo 
echo "launch process group: 0"
pids=()
echo "  cargo build"
cargo "build" "-p" "automotive-reference-system" "--release" > /dev/null 2>&1 &
pids+=("$!")

verify_success

echo 
echo "launch process group: 1"
pids=()
echo "  Intersection Output"
./target/release/automotive-reference-system "-e" "-n" "Intersection Output" "-s" "Euclidean Cluster Detector" "--interval-in-ms" "100" "actor" > /dev/null 2>&1 &
pids+=("$!")

echo "  Vehicle DBW System"
./target/release/automotive-reference-system "-e" "-n" "Vehicle DBW System" "-s" "Vehicle Interface" "--interval-in-ms" "100" "actor" > /dev/null 2>&1 &
pids+=("$!")

echo "  Behavior Planner"
./target/release/automotive-reference-system "-c" "100" "-e" "-n" "Behavior Planner" "-s" "Object Collision Estimator" "-s" "NDT Localizer" "-s" "Lanelet2Global Planner" "-s" "Lanelet2Map Loader" "-s" "Parking Planner" "-s" "Lane Planner" "--interval-in-ms" "250" "processing" "-o" "Behavior Planner" > /dev/null 2>&1 &
pids+=("$!")

echo "  Euclidean Cluster Detector"
./target/release/automotive-reference-system "-e" "-n" "Euclidean Cluster Detector" "-s" "Ray Ground Filter" "-s" "Euclidean Cluster Settings" "--interval-in-ms" "100" "processing" "-o" "Euclidean Cluster Detector" > /dev/null 2>&1 &
pids+=("$!")

echo "  Front Points Transformer"
./target/release/automotive-reference-system "-e" "-n" "Front Points Transformer" "-s" "Front Lidar Driver" "--interval-in-ms" "100" "processing" "-o" "Front Points Transformer" > /dev/null 2>&1 &
pids+=("$!")

echo "  Lane Planner"
./target/release/automotive-reference-system "-e" "-n" "Lane Planner" "-s" "Lanelet2Map Loader" "--interval-in-ms" "100" "processing" "-o" "Lane Planner" > /dev/null 2>&1 &
pids+=("$!")

echo "  Lanelet2Global Planner"
./target/release/automotive-reference-system "-e" "-n" "Lanelet2Global Planner" "-s" "NDT Localizer" "-s" "Visualizer" "--interval-in-ms" "100" "processing" "-o" "Lanelet2Global Planner" > /dev/null 2>&1 &
pids+=("$!")

echo "  Lanelet2Map Loader"
./target/release/automotive-reference-system "-e" "-n" "Lanelet2Map Loader" "-s" "Lanelet2Global Planner" "-s" "Lanelet2Map" "--interval-in-ms" "100" "processing" "-o" "Lanelet2Map Loader" > /dev/null 2>&1 &
pids+=("$!")

echo "  MPC Controller"
./target/release/automotive-reference-system "-e" "-n" "MPC Controller" "-s" "Behavior Planner" "--interval-in-ms" "100" "processing" "-o" "MPC Controller" > /dev/null 2>&1 &
pids+=("$!")

echo "  NDT Localizer"
./target/release/automotive-reference-system "-e" "-n" "NDT Localizer" "-s" "Voxel Grid Downsampler" "-s" "Point Cloud Map Loader" "--interval-in-ms" "100" "processing" "-o" "NDT Localizer" > /dev/null 2>&1 &
pids+=("$!")

echo "  Object Collision Estimator"
./target/release/automotive-reference-system "-e" "-n" "Object Collision Estimator" "-s" "Euclidean Cluster Detector" "--interval-in-ms" "100" "processing" "-o" "Object Collision Estimator" > /dev/null 2>&1 &
pids+=("$!")

echo "  Parking Planner"
./target/release/automotive-reference-system "-e" "-n" "Parking Planner" "-s" "Lanelet2Map Loader" "--interval-in-ms" "100" "processing" "-o" "Parking Planner" > /dev/null 2>&1 &
pids+=("$!")

echo "  Point Cloud Fusion"
./target/release/automotive-reference-system "-e" "-n" "Point Cloud Fusion" "-s" "Front Points Transformer" "-s" "Rear Points Transformer" "--interval-in-ms" "100" "processing" "-o" "Point Cloud Fusion" > /dev/null 2>&1 &
pids+=("$!")

echo "  Point Cloud Map Loader"
./target/release/automotive-reference-system "-e" "-n" "Point Cloud Map Loader" "-s" "Point Cloud Map" "--interval-in-ms" "100" "processing" "-o" "Point Cloud Map Loader" > /dev/null 2>&1 &
pids+=("$!")

echo "  Ray Ground Filter"
./target/release/automotive-reference-system "-e" "-n" "Ray Ground Filter" "-s" "Point Cloud Fusion" "--interval-in-ms" "100" "processing" "-o" "Ray Ground Filter" > /dev/null 2>&1 &
pids+=("$!")

echo "  Rear Points Transformer"
./target/release/automotive-reference-system "-e" "-n" "Rear Points Transformer" "-s" "Rear Lidar Driver" "--interval-in-ms" "100" "processing" "-o" "Rear Points Transformer" > /dev/null 2>&1 &
pids+=("$!")

echo "  Vehicle Interface"
./target/release/automotive-reference-system "-e" "-n" "Vehicle Interface" "-s" "MPC Controller" "-s" "Behavior Planner" "--interval-in-ms" "100" "processing" "-o" "Vehicle Interface" > /dev/null 2>&1 &
pids+=("$!")

echo "  Voxel Grid Downsampler"
./target/release/automotive-reference-system "-e" "-n" "Voxel Grid Downsampler" "-s" "Point Cloud Fusion" "--interval-in-ms" "100" "processing" "-o" "Voxel Grid Downsampler" > /dev/null 2>&1 &
pids+=("$!")

echo "  Euclidean Cluster Settings"
./target/release/automotive-reference-system "-e" "-n" "Euclidean Cluster Settings" "-s" "Euclidean Cluster Settings" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1 &
pids+=("$!")

echo "  Lidar"
./target/release/automotive-reference-system "-e" "-n" "Lidar" "-s" "Rear Lidar Driver" "-s" "Front Lidar Driver" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1 &
pids+=("$!")

echo "  Point Cloud Map"
./target/release/automotive-reference-system "-e" "-n" "Point Cloud Map" "-s" "Point Cloud Map" "-s" "Lanelet2Map" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1 &
pids+=("$!")

echo "  Visualizer"
./target/release/automotive-reference-system "-e" "-n" "Visualizer" "-s" "Visualizer" "--interval-in-ms" "100" "sensor" > /dev/null 2>&1 &
pids+=("$!")

verify_success


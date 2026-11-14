#!/bin/bash --login
#SBATCH -N1 -n1
#SBATCH --partition=cpu-genoa
#SBATCH --cpus-per-task=192
#SBATCH --time=01:00:00
#SBATCH --job-name=run-raptor
#SBATCH --output=logs/mock-routing_%j.log

nodes=($(scontrol show hostnames "$SLURM_NODELIST"))
server_node="${nodes[0]}"

(
srun -N1 -n1 -w "$server_node" \
    --output="$LOG_DIR/A_B_server.log" \
    make run
) &

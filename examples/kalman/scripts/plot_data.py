import numpy as np
import matplotlib.pyplot as plt
import matplotlib as mpl
import pandas as pd
import os
from sklearn.metrics import mean_squared_error, mean_absolute_error

# colors = mpl.colormaps["Set1"].colors
colors = mpl.colormaps["inferno"].colors
# colors = mpl.colormaps["magma"].colors
# colors = mpl.colormaps["plasma"].colors
# colors = mpl.colormaps["tab10"].colors
columns = ["roll", "pitch", "yaw"]
measure_columns = ["wx", "wy", "wz", "ax", "ay", "az"]
angles = ["$\phi$", "$\\theta$", "$\psi$"]
directions = ["$x$", "$y$", "$z$"]

clean_path = os.path.dirname(os.path.realpath(__file__))
true_values = clean_path + "/../records/true_position.csv"
pred_values = clean_path + "/../records/pred_position.csv"
noisy_measurements = clean_path + "/../records/measurements.csv"
measurements = clean_path + "/../records/clean_measurements.csv"

df_in = pd.read_csv(true_values)
df_out = pd.read_csv(pred_values)
measure = pd.read_csv(measurements)
noise_measure = pd.read_csv(noisy_measurements)

f0, ax = plt.subplots(
    1, 3, sharex=True, sharey=True, figsize=(16, 4), layout="constrained"
)

for idx in range(3):
    ax[idx].set_title("True Orientation in {} (rad)".format(angles[idx]))
    ax[idx].plot(df_in["t"], df_in[columns[idx]], color=colors[0])
f0.legend(["Ground Truth", "Estimation"], ncol=2, loc="outside lower center")

f1, ax = plt.subplots(
    1, 3, sharex=True, sharey=True, figsize=(16, 4), layout="constrained"
)

for idx in range(3):
    ax[idx].set_title(" Orientation in {} (rad)".format(angles[idx]))
    ax[idx].plot(
        df_in["t"],
        df_in[columns[idx]],
        color=colors[32],
        linestyle=(0, (5, 2)),
    )
    ax[idx].plot(df_in["t"], df_out[columns[idx]], color=colors[150])
f1.legend(["Ground Truth", "Estimation"], ncol=2, loc="outside lower center")

f2, ax = plt.subplots(2, 3, sharex=True, figsize=(16, 8), layout="constrained")

for idx in range(6):
    if idx < 3:
        ax[0, idx].set_title("Angular Rate {} (rad/s)".format(angles[idx]))
    else:
        nidx = idx - 3
        ax[1, nidx].set_title(
            "Linear Acceleration {} ($m/s^2$)".format(directions[nidx])
        )
    print(idx // 3, idx % 3)
    ax[idx // 3, idx % 3].plot(
        df_in["t"],
        noise_measure[measure_columns[idx]],
        color=colors[32],
        linestyle=(0, (1, 1)),
        linewidth=4,
    )
    ax[idx // 3, idx % 3].plot(
        df_in["t"],
        measure[measure_columns[idx]],
        color=colors[240],
    )
f2.legend(
    ["Noisy Measurements", "Ideal Measurements"], ncol=2, loc="outside lower center"
)

for val in columns:
    # print(np.square(df_in[val]-df_out[val]).mean())
    mse = mean_squared_error(df_in[val], df_out[val])
    mae = mean_absolute_error(df_in[val], df_out[val])
    print(val, " MSE: ", mse)
    print(val, " MAE: ", mae)

plt.show()

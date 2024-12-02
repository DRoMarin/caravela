import numpy as np
import matplotlib.pyplot as plt
import matplotlib as mpl
import pandas as pd

# colors = mpl.colormaps["Set1"].colors
colors = mpl.colormaps["tab10"].colors
columns = ["roll", "pitch", "yaw"]
measure_columns = ["wx", "wy", "wz", "ax", "ay", "az"]
angles = ["$\phi$", "$\\theta$", "$\psi$"]
directions = ["$x$", "$y$", "$z$"]
true_values = "../records/true_position.csv"
pred_values = "../records/pred_position.csv"
measurements = "../records/measurements.csv"
df_in = pd.read_csv(true_values)
df_out = pd.read_csv(pred_values)
measure = pd.read_csv(measurements)

f0, ax = plt.subplots(3, 3, sharex=True, sharey=True, figsize=(16, 9))

for idx in range(3):

    ax[0, idx].set_title("True Orientation {}".format(angles[idx]))
    ax[0, idx].plot(df_in["t"], df_in[columns[idx]], color=colors[idx])
    # ax[0, idx].set_ylim([-2, 2])

    ax[1, idx].set_title("Estimated Orientation {}".format(angles[idx]))
    ax[1, idx].plot(df_in["t"], df_out[columns[idx]], color=colors[3 + idx])
    # ax[1, idx].set_ylim([-2, 2])

    ax[2, idx].set_title("Difference {}".format(angles[idx]))
    ax[2, idx].plot(df_in["t"], df_in[columns[idx]], color=colors[idx])
    ax[2, idx].plot(df_in["t"], df_out[columns[idx]], color=colors[3 + idx])
    # ax[2, idx].set_ylim([-2, 2])

# print((df_in[columns[0]] - df_out[columns[0]]))


f1, ax = plt.subplots(2, 3, sharex=True, figsize=(16, 9))
for idx in range(6):
    if idx < 3:
        ax[0, idx].set_title("Noisy Angular Rate {}".format(angles[idx]))
    else:
        nidx = idx - 3
        ax[1, nidx].set_title("Noisy Linear Acceleration {}".format(directions[nidx]))
    print(idx // 3, idx % 3)
    ax[idx // 3, idx % 3].plot(
        df_in["t"], measure[measure_columns[idx]], color=colors[idx]
    )

plt.show()

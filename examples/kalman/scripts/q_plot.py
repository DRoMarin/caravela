from scipy.spatial.transform import Rotation as R
from matplotlib.animation import FuncAnimation
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import time


def rotate_vector(vector, rotation_matrix):
    return np.dot(rotation_matrix, vector)


def calculate_endpoint(start, a, b, c):
    rotation_matrix = R.from_euler(
        "xyz",
        [a, b, c],
    ).as_matrix()
    u1 = np.array([1, 0, 0])
    u2 = np.array([0, 1, 0])
    u3 = np.array([0, 0, 1])
    v1 = rotate_vector(u1, rotation_matrix)
    v2 = rotate_vector(u2, rotation_matrix)
    v3 = rotate_vector(u3, rotation_matrix)
    # print(v1, v2, v3)
    return v2


x = 0
y = 0
z = 0

start_point = np.array([x, y, z])
data = []

df = pd.read_csv("../records/true_position.csv")

for index, row in df.iterrows():
    # print(row["roll"], row["pitch"], row["yaw"])
    # end_point = [row["roll"], row["pitch"], 0]
    end_point = calculate_endpoint(start_point, row["roll"], row["pitch"], 0)
    # print(row["t"])
    # time.sleep(0.005)
    data.append(end_point)

fig, ax = plt.subplots(subplot_kw=dict(projection="3d"))

quiver = ax.quiver([], [], [], [], [], [])

ax.set_xlim(-1, 1)
ax.set_ylim(-1, 1)
ax.set_zlim(-1, 1)


def update(i):
    global quiver
    quiver.remove()
    #if (i % 100) == 0:
    print(i)
    quiver = ax.quiver(0, 0, 0, data[i][0], data[i][1], data[i][2], linewidth=3)

ani = FuncAnimation(fig, update, interval=1, frames=len(data), repeat=False)
#plt.show()
ani.save('../quiver_test_ani.mp4', writer='ffmpeg')
plt.close()

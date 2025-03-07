import numpy as np
import pandas as pd
from scipy.signal import savgol_filter
import argparse


def calc_noise(signal, std_deviation=1, mean_noise=0):
    sig_avg_mag = np.mean(abs(signal))
    print("mean: ", mean_noise, " variance: {:.5e}".format(std_deviation**2))
    noise_signal = np.random.normal(mean_noise, std_deviation, len(signal))
    return noise_signal


def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-s",
        "--sampling-frequency",
        help="Sampling frequency of the signal in Hz",
        default=1000,
    )

    parser.add_argument("-f", "--attitude-file", help="", required=True)

    parser.add_argument(
        "-a",
        "--accelerometer",
        help="",
        default="0.1",
    )
    parser.add_argument(
        "-g",
        "--gyroscope",
        help="",
        default="0.001",
    )
    args = parser.parse_args()
    df = pd.read_csv(args.attitude_file)
    fs = float(args.sampling_frequency)
    std_acc = float(args.accelerometer)
    std_gyro = float(args.gyroscope)

    return (fs, df, std_acc, std_gyro)


def normalize(signal):
    signal = (
        ((signal - signal.min()) / (signal.max() - signal.min()) - 0.5)
        * 0.975 * np.pi/2
    )
    return signal


def smooth(signal):
    filtered = savgol_filter(signal, 200, 2, mode="nearest")
    return filtered


(fs, input_df, std_acc, std_gyro) = get_args()
signal_phi = input_df["phi"]
signal_theta = input_df["theta"]
signal_psi = input_df["psi"]

signal_phi = normalize(smooth(signal_phi))
signal_theta = normalize(smooth(signal_theta))
signal_psi = normalize(smooth(signal_psi))

timeline, dx = np.linspace(
    0, len(signal_phi) / fs, int(len(signal_phi)), endpoint=False, retstep=True
)

w_phi = np.gradient(signal_phi, dx)
w_theta = np.gradient(signal_theta, dx)
w_psi = np.gradient(signal_psi, dx)

print("Angular Rate")
w_phi_noise = w_phi + calc_noise(w_phi, std_gyro)
w_theta_noise = w_theta + calc_noise(w_theta, std_gyro)
w_psi_noise = w_psi + calc_noise(w_psi, std_gyro)

g = 9.81

a_x = g * np.sin(signal_theta)
a_y = -g * np.cos(signal_theta) * np.sin(signal_phi)
a_z = -g * np.cos(signal_theta) * np.cos(signal_phi)

print("Linear Acceleration")
a_x_noise = a_x + calc_noise(a_x, std_acc)
a_y_noise = a_y + calc_noise(a_y, std_acc)
a_z_noise = a_z + calc_noise(a_z, std_acc)

measure_df = pd.DataFrame(
    np.transpose(
        [
            timeline,
            a_x,
            a_y,
            a_z,
            w_phi,
            w_theta,
            w_psi,
        ]
    )
)
noisy_measure_df = pd.DataFrame(
    np.transpose(
        [
            timeline,
            a_x_noise,
            a_y_noise,
            a_z_noise,
            w_phi_noise,
            w_theta_noise,
            w_psi_noise,
        ]
    )
)

print("Measurements frame: ", measure_df.shape)
measure_df.columns = ["t", "ax", "ay", "az", "wx", "wy", "wz"]
measure_df.to_csv(
    "/home/drojasm/Desktop/CARAVEL/caravel-fix/examples/kalman/records/clean_measurements.csv",
    float_format="%.5f",
    index=False,
)

noisy_measure_df.columns = ["t", "ax", "ay", "az", "wx", "wy", "wz"]
noisy_measure_df.to_csv(
    "/home/drojasm/Desktop/CARAVEL/caravel-fix/examples/kalman/records/measurements.csv",
    float_format="%.5f",
    index=False,
)

position_df = pd.DataFrame(
    np.transpose(
        [
            timeline,
            signal_phi,
            signal_theta,
            signal_psi,
        ]
    )
)

print("True positions frame: ", position_df.shape)
position_df.columns = ["t", "roll", "pitch", "yaw"]
position_df.to_csv(
    "/home/drojasm/Desktop/CARAVEL/caravel-fix/examples/kalman/records/true_position.csv",
    float_format="%.4f",
    index=False,
)

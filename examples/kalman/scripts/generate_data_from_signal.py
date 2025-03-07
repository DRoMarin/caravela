import numpy as np
import pandas as pd
from scipy.signal import savgol_filter
import argparse


def calc_noise(signal, target_snr_db=20, mean_noise=0):
    # sig_avg_mag = np.mean(signal)
    sig_avg_mag = np.mean(abs(signal))
    # sig_avg_mag = np.max(abs(signal))*np.sqrt(2)
    sig_avg_db = 10 * np.log10(sig_avg_mag)
    noise_avg_db = sig_avg_db - target_snr_db
    noise_avg_mag = 10 ** (noise_avg_db / 10)
    print("mean: ", mean_noise, " variance: {:.5e}".format(noise_avg_mag))
    noise_signal = np.random.normal(mean_noise, np.sqrt(noise_avg_mag), len(signal))
    return noise_signal


def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-s",
        "--sampling-frequency",
        help="Sampling frequency of the signal in Hz",
        default=1000,
    )

    parser.add_argument(
        "-a",
        "--attitude-file",
        help="",
        default="",
    )

    parser.add_argument(
        "-n",
        "--target-snr",
        help="Target SNR of the measurement signals (gyro/accel) in dB",
        default=20,
    )
    args = parser.parse_args()
    df = pd.read_csv(args.attitude_file)
    fs = float(args.sampling_frequency)
    snr = float(args.target_snr)

    return (fs, df, snr)


def normalize(signal):
    signal = (
        ((signal - signal.min()) / (signal.max() - signal.min()) - 0.5)
        * np.pi
        / 2.0
        * 0.9
    )
    return signal


def smooth(signal):
    filtered = savgol_filter(signal, 200, 2, mode="nearest")
    print(filtered)
    return filtered


(fs, input_df, snr) = get_args()
signal_phi = input_df["phi"]
signal_theta = input_df["theta"]
signal_psi = input_df["psi"]

signal_phi = normalize(smooth(signal_phi))
signal_theta = normalize(smooth(signal_theta))
signal_psi = normalize(smooth(signal_psi))

timeline, dx = np.linspace(
    0, len(signal_phi) / fs, int(len(signal_phi)), endpoint=False, retstep=True
)

print(timeline, dx)

# dx = 2 * np.pi * f / (fs)
# dx = 1
w_phi = np.gradient(signal_phi, dx)
w_theta = np.gradient(signal_theta, dx)
w_psi = np.gradient(signal_psi, dx)

print("Angular Rate")
w_phi_noise = w_phi + calc_noise(w_phi, snr)
w_theta_noise = w_theta + calc_noise(w_theta, snr)
w_psi_noise = w_psi + calc_noise(w_psi, snr)

g = 9.81

a_x = g * np.sin(signal_theta)
a_y = -g * np.cos(signal_theta) * np.sin(signal_phi)
a_z = -g * np.cos(signal_theta) * np.cos(signal_phi)

print("Linear Acceleration")
a_x_noise = a_x + calc_noise(a_x, snr)
a_y_noise = a_y + calc_noise(a_y, snr)
a_z_noise = a_z + calc_noise(a_z, snr)

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

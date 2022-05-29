import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict

saved = [
    "decode_0",
    "decode_1",
    "decode_2",
    "decode_3",
    "decode_4",
    "decode_5",
    "decode_6",
    "decode_7",
    "decode_8",
    "decode_9",
    "decode_10",
    "decode_11",
    "decode_12",
    "decode_13",
    "decode_14",
    "decode_15",
    "addr_0",
    "addr_1",
    "addr_2",
    "addr_3",
    "addr_b_0",
    "addr_b_1",
    "addr_b_2",
    "addr_b_3",
]

plot = [f"decode_{i}" for i in range(16)]
# plot = ["decode_9"]


def read_data(f):
    data = defaultdict(lambda: [])
    for line in f.readlines():
        values = line.split()
        ctr = 0
        for key in saved:
            if ctr == 0:
                data["time"].append(float(values[ctr]))
            ctr += 1
            data[key].append(float(values[ctr]))
            ctr += 1
    return {k: np.array(v) for k, v in data.items()}


def read_test_data():
    with open("./decoder_16.dat") as f:
        return read_data(f)


def plot_data(data):
    plt.figure()
    for key in plot:
        plt.plot(data["time"], data[key])
    plt.legend(plot)
    plt.savefig("decoder_16.png")


def analyze_data(data):
    t = data["time"]
    for i in range(1, 16):
        tr = rise_time(t, data[f"decode_{i}"])
        print(f"decode_{i}: {tr*1e12}ps")


def rise_time(t, v, low_thresh=0.2, high_thresh=0.8):
    maximum = np.max(v)
    minimum = np.min(v)

    # Normalize v
    vnorm = (v - minimum) / (maximum - minimum)
    t_high = (vnorm >= high_thresh).nonzero()[0][0]
    t_low = (vnorm[:t_high] <= low_thresh).nonzero()[0][-1]

    assert t_high > t_low

    return t[t_high] - t[t_low]


if __name__ == "__main__":
    data = read_test_data()
    analyze_data(data)
    plot_data(data)
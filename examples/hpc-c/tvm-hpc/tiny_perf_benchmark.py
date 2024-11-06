#!/usr/bin/env python3

import argparse
import os
from pathlib import Path
import pandas as pd
from PIL import Image
import numpy as np
import librosa as lb
import re
import serial
from sklearn.metrics import accuracy_score, confusion_matrix, classification_report
from matplotlib import pyplot as plt
import tensorflow as tf
import time

UART = '/tmp/uart0'
ROOT_PATH = Path(__file__).parents[0]
DATA_DIR = ROOT_PATH / "dev_data"
KWS_DATA_DIR = DATA_DIR / "kws01"
VWW_DATA_DIR = DATA_DIR / "vw_coco2014_96"
VWW_NON_PERSON_DATA_DIR =  VWW_DATA_DIR / "non_person"
VWW_PERSON_DATA_DIR = VWW_DATA_DIR / "person"
IC_DATA_DIR = DATA_DIR / "cifar-10-batches-py"
AD_DATA_DIR = DATA_DIR / "ToyCar" / "test"

import numpy as np

def print_matrix(arr, format_type='signed'):
    for row in arr:
        for elem in row:
            if format_type == 'signed':
                print(f"{int(elem):d}", end="\t")  # Signed decimal
            elif format_type == 'unsigned':
                print(f"{int(elem) & 0xFFFFFFFF:d}", end="\t")  # Unsigned decimal
            elif format_type == 'hex':
                print(f"{int(elem) & 0xFFFFFFFF:08x}", end="\t")  # Hexadecimal
            else:
                raise ValueError("Invalid format_type. Use 'signed', 'unsigned', or 'hex'.")
        print()


def accuracy_report(gt, prediction):
    print("Accuracy: {:.3f}".format(accuracy_score(gt, prediction)))
    print("Confusion matrix:\n{}".format(confusion_matrix(gt, prediction)))
    print(classification_report(gt, prediction))

def send_stimulus(data, label=None):
    print("Writing {} bytes as stimulus...".format(len(data)))
    if label is not None:
        print("Expected label: {}".format(label))
    ser = serial.Serial(UART, 9600)
    print("Sending stimulus...")
    ser.write(bytes(data))
    ser.close()

def wait_for_result():
    print("Waiting for results...")
    ser = serial.Serial(UART,  9600)
    output = ser.readline()
    while output != b'Prediction:\n':
        output = ser.readline()
    output = ser.readline()
    ser.close()
    output = bytearray(output)
    results = []
    for x in output:
        results.append(((x & 0xff) ^ 0x80) - 0x80) # Append signed
    results = results[:-1] # Remove line break
    print(results)
    print("Predicted class: {}".format(np.argmax(results)))
    print("\n")
    return results

def read_kws_file(path):
    with open(path, mode="rb") as file:
        content = file.read()
    return content

def run_kws():
    df = pd.read_csv(KWS_DATA_DIR / "y_labels.csv", names=["filename", "no_classes", "class"])
    print("Input shape: ( ,{})".format(len(read_kws_file(KWS_DATA_DIR / df["filename"][0]))))

    predictions = []
    for (i, filename) in enumerate(df["filename"]):
        data = read_kws_file(KWS_DATA_DIR / filename)
        send_stimulus(data, df["class"][i])
        predictions.append(np.argmax(wait_for_result()))

    accuracy_report(df["class"], predictions)

def get_kws_stimulus():
    df = pd.read_csv(KWS_DATA_DIR / "y_labels.csv", names=["filename", "no_classes", "class"])
    data = read_kws_file(KWS_DATA_DIR / df["filename"][0])
    print("Expected label:", df["class"][0])
    return data

def read_vww_file(path):
    #Image loading and preprocessing
    image = tf.io.read_file(str(path))
    image = tf.image.decode_jpeg(image, channels=3)
    image = tf.image.resize(image, [96,96])
    image = tf.cast(image, tf.uint8)
    byte_array = image.numpy()
    return byte_array

def run_vww():
    items = os.listdir(VWW_NON_PERSON_DATA_DIR)
    non_persons = [item for item in items if os.path.isfile(os.path.join(VWW_NON_PERSON_DATA_DIR, item)) and item.startswith("COCO_val")]

    items = os.listdir(VWW_PERSON_DATA_DIR)
    persons = [item for item in items if os.path.isfile(os.path.join(VWW_PERSON_DATA_DIR, item)) and item.startswith("COCO_val")]

    print("Number of non_persons", len(non_persons))
    print("Number of persons", len(persons))
    print("Input shape: ", np.shape(read_vww_file(VWW_NON_PERSON_DATA_DIR / non_persons[0])))

    # Generate ground truth array
    gt_non = [0 for _ in non_persons]
    gt_yes = [1 for _ in persons]
    gt = gt_non + gt_yes

    predictions = []
    for x in non_persons:
        data = read_vww_file(VWW_NON_PERSON_DATA_DIR / x)

        send_stimulus(data.tobytes())
        predictions.append(wait_for_result())

    for x in persons:
        data = read_vww_file(VWW_PERSON_DATA_DIR / x)
        send_stimulus(data.tobytes())
        predictions.append(wait_for_result())

    accuracy_report(gt, predictions)

def get_vww_stimulus():
    items = os.listdir(VWW_NON_PERSON_DATA_DIR)
    non_persons = [item for item in items if os.path.isfile(os.path.join(VWW_NON_PERSON_DATA_DIR, item)) and item.startswith("COCO_val")]
    data = read_vww_file(VWW_NON_PERSON_DATA_DIR / non_persons[0])
    print("Expected label: 1")
    return data.tobytes()


def run_ic():
    import pickle
    with open(IC_DATA_DIR / "test_batch", "rb") as file:
        data = pickle.load(file, encoding='bytes')
    print("Input shape: {}".format(np.shape(data[b'data'][0])))

    predictions = []
    for (i, image) in enumerate(data[b'data']):
        #FROM CHW to HWC
        print("Running inference for image {}/{}".format(i, len(data[b'data'])))
        image = np.reshape(image, (3, 32, 32))
        image = np.rollaxis(image, 0, 3)
        image = image - 128
        image = np.reshape(image, (3072))
        label = data[b'labels'][i]
        send_stimulus(image.tobytes(), label)

        # Wait for inference result
        predictions.append(np.argmax(wait_for_result()))
        pass

    # Evaluate predictions
    accuracy_report(data[b'labels'], predictions)

def get_ic_stimulus():
    import pickle
    with open(IC_DATA_DIR / "test_batch", "rb") as file:
        data = pickle.load(file, encoding='bytes')
    print("Expected label:", data[b'labels'][0])
    return data[b'data'][0].tobytes()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-b", "--benchmark", required=True)
    opts = parser.parse_args()
    if opts.benchmark == "kws":
        run_kws()
    elif opts.benchmark == "vww":
        run_vww()
    elif opts.benchmark == "ic":
        run_ic()
    else:
        print("Bad benchmark! Available benchmarks are: kws, vww, ic")

if __name__ == "__main__":
    main()

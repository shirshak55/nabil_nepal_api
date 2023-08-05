# Nabil Bank Sample API Enhancement

## Introduction

The Nabil Bank Sample API Enhancement project aims to address limitations in the existing app's statement retrieval
functionality. Currently, the app lacks the capability to provide accurate and comprehensive monthly statements.
Additionally, there is no built-in option to retrieve statements for an entire year. To overcome these limitations, this
project introduces a solution through custom code implementation.

# Disclaimer

It is just a toy project. If you cannot read the source code, you cannot run this project or use the code for sure.

## Problem Statement

The existing Nabil Bank app falls short in delivering accurate and user-friendly statement retrieval, particularly on a
monthly and yearly basis. Users are unable to access statements effectively, leading to frustration and inconvenience.
In response, this project presents a code-based solution to enhance the app's statement retrieval capabilities.

## Solution

To address the shortcomings of the Nabil Bank app, a custom solution has been developed. The project employs the Frida
dynamic instrumentation toolkit to enhance the app's behavior. The following steps outline the setup process:

## Running

1. Just put your details in config.toml
2. Run `cargo run`

### Setting Up Frida

1. Copy the Frida server binary to the target device using the following command:

    ```sh
    adb push ./frida-server-$version-android-$arch /data/local/tmp/frida-server
    ```

Replace `$version` and `$arch` with the appropriate version and architecture details of the Frida server binary.

2. Enable root access on the target device using the command:

    ```sh
    adb root
    ```

3. Grant executable permissions to the Frida server binary:

    ```sh
    adb shell "chmod 755 /data/local/tmp/frida-server"
    ```

4. Start the Frida server on the device:

    ```sh
    adb shell "/data/local/tmp/frida-server &"
    ```

### Initiating Frida Script

1. Install the necessary Frida tools using pip3:

    ```sh
    pip3 install frida-tools
    ```

2. List the running processes on the device to identify the target app:

    ```sh
    frida-ps -U
    ```

3. Launch the Frida script with the target app's package name using the following command:

    ```sh
    frida --no-pause -U -l ./frida.js -f com.f1soft.nabilmbank
    ```

## Key Features

-   Improved Statement Retrieval: The enhanced app offers accurate and comprehensive statement retrieval, eliminating
    the inconvenience caused by the original limitations.

-   Monthly and Yearly Statements: Users can effortlessly obtain both monthly and yearly statements, providing a
    holistic view of their financial activities.

## Technologies Used

-   Frida: A dynamic instrumentation toolkit used to inject custom code into the app and modify its behavior.

-   HTTP Toolkit: It is man in the middle proxy.

## Conclusion

The Nabil Bank Sample API Enhancement project brings a viable solution to the statement retrieval limitations of the
existing app. By leveraging Frida and custom code implementation, users can now enjoy improved and more flexible access
to their financial statements, both on a monthly and yearly basis. This enhancement aims to provide a seamless and
user-friendly experience for Nabil Bank app users.

```

This should complete the README Markdown code based on your request. Feel free to copy and paste this continuation into your README file.
```

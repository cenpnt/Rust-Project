<h1 align="center">🖥️ System Monitoring Project 🖥️ </h1>

<h3 align="center">
Pannatat Sribusarakham 66011109

King Mongkut's Institute of Technology Ladkrabang</h3>

![all](https://github.com/cenpnt/Rust-Project/assets/139846503/929c9096-ed4e-410b-ae55-41946f214cce)


<details> 
<summary> Table of Contents </summary>

* [Installation](#installation)
* [Project Description](#project-description)
* [Key Features](#key-features)
    * [CPU Usage](#cpu-usage)
    * [Memory](#memory)
    * [Network](#network)
    * [Process](#process)
    * [Disk](#disk)
    * [Temperature](#temperature)
    * [Battery](#battery)
* [Run the Program](#run-the-program)
</details>

## Installation

>[!IMPORTANT]
> **Add `ratatui` `crossterm` `sysinfo` and `battery` as dependencies to your cargo.toml:**

```
cargo add ratatui crossterm sysinfo battery
```

```
- [dependencies]
    - sysinfo = "0.29.10"
    - crossterm = "0.25"
    - ratatui = "0.23.0"
    - battery = "0.7.8"
```

## Project Description

System Monitoring is a Text-based User Interface (TUI) that provides users with a thorough overview of the state of their system, including information about memory, CPU usage, network, and other aspects.

A dashboard displays the information, while System Monitoring provides an interactive menu with keypress actions to enhance interaction between users. For instance, by pressing specific keys (such as 'c' to visit the CPU usage page), users can quickly and simply navigate between different data pages, such as CPU usage, providing a straightforward and user-friendly experience.

Furthermore, the TUI interface is equipped to display bar graphs, line charts and more for visualizing data, like the average CPU usage over a specified time period.

## Key Features

- ### CPU Usage
  
     ![cpu](https://github.com/cenpnt/Rust-Project/assets/139846503/6122e2e2-6504-4e6d-94f7-fd273e932785)

    This section shows the following data:

    - CPU threads
    - The average of all threads
    - Bar graphs for each CPU
    - A line chart that presents an overview of the overall CPU usage
___

- ### Memory
     ![memory](https://github.com/cenpnt/Rust-Project/assets/139846503/bc2a8c57-e5ac-4ac9-84e5-a131e2a9414f)

    This section shows the following data:

    - Available memory
    - Used memory
    - Total memory
    - Gauge bar comparing between used memory and total memory displaying in percentage
---

* ### Network
   ![network](https://github.com/cenpnt/Rust-Project/assets/139846503/a79c692a-bcec-4e27-81f6-94c4de3d4f6e)

     
    This section shows the following data:

    - Received data
    - Transmitted data
    - Scroll bar for navigating through each element in the network.
---

* ### Process
  ![process](https://github.com/cenpnt/Rust-Project/assets/139846503/ae77410e-2c0e-4b02-b365-c97d5f277aef)

    This section shows the following data:

    - Process ID
    - Usage
    - Scroll bar for navigating through each element in process.

--- 

* ### Disk
  <img width="1641" alt="disk screenshot" src="https://github.com/cenpnt/Rust-Project/assets/139846503/45e62f28-d658-4585-8656-77552557f0a5">

    This section shows the following data:

    - Name
    - Type
    - Total Space
    - Used Space
    - Free Space
    - Gauge bar comparing between total space and used space

---

* ### Temperature
  ![temp](https://github.com/cenpnt/Rust-Project/assets/139846503/8d63193b-731b-41a1-bd9d-333206de0bc4)

    This section shows the following data:

    - Name
    - Temperature in Celsius
    - Bar graphs that compare the temperature of all the elements.

---

* ### Battery
  <img width="1641" alt="battery screenshot" src="https://github.com/cenpnt/Rust-Project/assets/139846503/b685950a-da18-4eb2-bd29-002f8f921e57">

    This section shows the following data:
    
    - State of battery (charging, discharging)
    - Battery percentage

## Run the Program
You can run the program simply by typing this command in the terminal:

        cargo run

To navigate through all the program's features, you can follow these steps:

- Press `c` to access the CPU section

- Press `m` to access the Memory section

- Press `n` to access the Network section

- Press `p` to access the Process section

- Press `d` to access the Disk section

- Press `t` to access the Temperature section

- Press `b` to access the Battery section

- Press `h` to return to the Home page

- Press `q` to quit the program

- Press `↑` to scroll up

- Press `↓` to scroll down

# Installation Instructions

Prerequisite: Have rust installed on the Pi

Start by installing the luz_optimon daemon.

```bash
cargo install --git https://github.com/Foulgaz3/luz_optimon.git
```

This should install the program to 

```bash
/home/${INSTALL_USER}/.cargo/bin/luz_optimon"
```

on the computer this was tested on, the user was set to `raspi`, so the program appeared in

`/home/raspi/.cargo/bin/luz_optimon`

## Systemd service 

I created this from the terminal, so that is how these instructions will be provided.

Change the directory to `/etc/systemd/system` and create a file called `luz_optima.service`


```bash
cd /etc/systemd/system
sudo vi luz_optima.service
```

The file should look like this:
```ini
[Unit]
Description=LuzOptima Application
After=network.target
StartLimitIntervalSec=10s
StartLimitBurst=50

[Service]
Type=simple
User=raspi
WorkingDirectory=/home/raspi/Desktop/WinterSetup
ExecStart=/home/raspi/.cargo/bin/luz_optimon /home/raspi/Desktop/WinterSetup/schedule_file.json
Restart=on-failure
Environment="PATH=/home/raspi/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"8

[Install]
WantedBy=multi-user.target
```

now save and run 

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now luz_optima.service
systemctl status luz_optima.service
```
and make sure the service is active and running.

## Systemd watcher

Create another file `luz_optima-reload.service`

```ini
[Unit]
Description=Restart luz_optimon when schedule file changes

[Service]
Type=oneshot
ExecStart=/bin/systemctl restart luz_optima.service
```

and another file called `luz_optima-reload.path`

```ini
[Unit]
Description=Watch schedule_file.json for changes

[Path]
PathModified=/home/raspi/Desktop/WinterSetup/schedule_file.json
Units=luz_optima-reload.service

[Install]
WantedBy=multi-user.target
```

now save both files and run

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now luz_optima-reload.path
```

## LED Cron job

First, enable the pigpiod daemon to run at startup

`sudo systemctl enable --now pigpiod.service`

start up a new terminal and type
```bash
crontab -e
```

Then add a new line to the job that looks like this:
```ini
*/5 * * * * /usr/bin/python3 /home/raspi/Desktop/WinterSetup/update_leds.py
```

This will run the update script once every 5 minutes.
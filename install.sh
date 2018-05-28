#! /bin/bash

URL=$(curl https://api.github.com/repos/kogai/sync-dir/releases/latest | grep 'browser_download_url' | grep -i "$(uname)" | cut -d\" -f4)
FILE=$(echo $URL | sed 's/.*\(sync-dir-.*\).tar.gz/\1/')

curl -L "$URL" > "/tmp/$FILE.tar.gz"
mkdir -p "/tmp/$FILE"
tar -xvf "/tmp/$FILE.tar.gz" -C "/tmp/$FILE"
mkdir -p ~/bin
mv "/tmp/$FILE/bin/$(uname)/sync-dir" ~/bin

if [ "$(uname)" = "Linux" ]; then
  sudo cp ~/bin/sync-dir.service /etc/systemd/system/sync-dir.service
  cat <<EOF
Enable sync-dir daemon

$ systemctl service enable sync-dir
EOF
else
  cat <<EOF
Enable sync-dir daemon

$ launchctl enable sync-dir
EOF
fi


#!/bin/sh

if [ -z "$REALM_ID" ]; then
  read -p "Enter a Tozny realm ID (e.g. sid_74a40187e2790): " REALM_ID
fi
if [ -z "$USER_ID" ]; then
  read -p "Enter a Tozny user ID (e.g. sid_c233df00c07b9): " USER_ID
fi

SCRIPT=$(readlink -f "$0")
SCRIPTPATH=$(dirname "$SCRIPT")

cd "$SCRIPTPATH"
docker build -t tozny-pam-demo .

CONTAINER=$(docker run -e "REALM_ID=$REALM_ID" -e "USER_ID=$USER_ID" -P -d tozny-pam-demo)

echo ""
echo "To connect to the demo ssh server:"
echo ""
echo "    $ ssh -p $(docker port "$CONTAINER" 22 | cut -d: -f2) gregory@localhost"
echo ""
echo "Type Ctrl-c to quit."
echo ""

cleanup() {
  echo "Stopping server..."
  docker stop "$CONTAINER" > /dev/null
  exit
}

trap cleanup INT TERM

while "true"; do
  sleep 1000
done;

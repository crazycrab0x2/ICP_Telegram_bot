#!/usr/bin/env bash

canister_id="$(jq -r .telegram.ic < canister_ids.json)"

if [ -z "$canister_id" ]; then
  echo "Could not read canister id for canister \"telegram\" from ./canister_ids"
  exit 1
fi

token="$(cat token)"
if [ -z "$token" ]; then
  echo "Could not read file ./token"
  exit 1
fi


curl "https://api.telegram.org/bot7964736455:AAEqFUH7-eCOUcKmMK4DX4N8qE5gIU3Cbdg/setWebhook?url=https://edrd5-rqaaa-aaaab-qaafq-cai.raw.icp0.io/webhook/7964736455:AAEqFUH7-eCOUcKmMK4DX4N8qE5gIU3Cbdg"

curl "https://api.telegram.org/bot7964736455:AAEqFUH7-eCOUcKmMK4DX4N8qE5gIU3Cbdg/setWebhook?url=https://iw7nm-6qaaa-aaaak-ao7ta-cai.raw.icp0.io/webhook/7964736455:AAEqFUH7-eCOUcKmMK4DX4N8qE5gIU3Cbdg"

CANISTERNAME=ICP_GPT_bot_backend

# Path to your config JSON file
config_json="config.json"

ADMIN=$(jq -r '.admin' "$config_json")
TOKEN=$(jq -r '.token' "$config_json")
MODEL=$(jq -r '.model' "$config_json")
PROMPT=$(jq -r '.prompt' "$config_json")

# Path to your canister Ids JSON file
canister_ids_json="canister_ids.json"

# Read the 'ic' value from the 'ICP_GPT_bot_backend' object
CANISTERID=$(jq -r '.ICP_GPT_bot_backend.ic' "$canister_ids_json")

# Check if jq was able to extract the value
if [[ ! -f "$canister_ids_json" ]]; then
    dfx deploy $CANISTERNAME --ic --argument '(record { model = $MODEL; token = $TOKEN; admin = $ADMIN; prompt = $PROMPT; })'
fi

dfx deploy $CANISTERNAME --ic --argument '(record { model = $MODEL; token = $TOKEN; admin = $ADMIN; prompt = $PROMPT; })' --mode upgrade

CANISTERID=$(jq -r '.ICP_GPT_bot_backend.ic' "$canister_ids_json")

# Set telegram bot web hook to canister
curl "https://api.telegram.org/bot$TOKEN/setWebhook?url=https://$CANISTERID.raw.icp0.io/webhook/$TOKEN"
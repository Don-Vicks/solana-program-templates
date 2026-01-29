/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/anchor_reinitialization.json`.
 */
export type AnchorReinitialization = {
  "address": "8tWvwysVozpvXRYJDngfiGw1HpgcNVZYx3T8ozeAWs2Q",
  "metadata": {
    "name": "anchorReinitialization",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "initializeInsecure",
      "discriminator": [
        66,
        109,
        130,
        167,
        42,
        38,
        232,
        10
      ],
      "accounts": [
        {
          "name": "user",
          "docs": [
            "The caller usually creates the account via SystemProgram before calling this.",
            "But if they call it TWICE, we just overwrite."
          ],
          "writable": true
        }
      ],
      "args": [
        {
          "name": "id",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initializeSecure",
      "discriminator": [
        22,
        242,
        50,
        101,
        87,
        199,
        204,
        53
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "id",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "user",
      "discriminator": [
        159,
        117,
        95,
        227,
        239,
        151,
        58,
        236
      ]
    }
  ],
  "types": [
    {
      "name": "user",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "id",
            "type": "u64"
          }
        ]
      }
    }
  ]
};

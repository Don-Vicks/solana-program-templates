/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/anchor_arbitrary_cpi.json`.
 */
export type AnchorArbitraryCpi = {
  "address": "79F2iKGQy1QqvVVVGohwpLy4XVRtNfzYjvnrD4T6Cb1e",
  "metadata": {
    "name": "anchorArbitraryCpi",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "cpiInsecure",
      "discriminator": [
        25,
        85,
        187,
        42,
        70,
        24,
        108,
        229
      ],
      "accounts": [
        {
          "name": "source",
          "writable": true
        },
        {
          "name": "destination",
          "writable": true
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "tokenProgram"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "cpiSecure",
      "discriminator": [
        20,
        92,
        119,
        17,
        28,
        89,
        199,
        87
      ],
      "accounts": [
        {
          "name": "source",
          "docs": [
            "Actually, if we use anchor_spl::token::transfer, it will fail if accounts are invalid for the token program.",
            "But typically we should use Account<'info, TokenAccount>. Keeping it simple for demo."
          ],
          "writable": true
        },
        {
          "name": "destination",
          "writable": true
        },
        {
          "name": "authority",
          "signer": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ]
};

# Tools

Every tool is annotated read-only and maps to one HyperBilling Open API request.

| Tool | HyperBilling request | Purpose |
| --- | --- | --- |
| `hyperbilling_list_linked_accounts` | `GET /v1/search/linkedaccount` | Linked accounts the caller can query |
| `hyperbilling_list_tag_keys` | `POST /v1/search/tag` | Cost allocation tag keys per linked account |
| `hyperbilling_search_billing` | `POST /v1/search/billing` | Custom period, grouping, measures, region and service filters, pagination |
| `hyperbilling_get_recent_billing` | `POST /v2/search/billing/{view}` | Seven fixed recent reports, selected by `view` |

## Call order

1. `hyperbilling_list_linked_accounts` returns the account IDs every other tool needs.
2. `hyperbilling_list_tag_keys` returns the tag keys, when the query groups by a tag.
3. `hyperbilling_search_billing` runs the query. A non-empty `NextDataToken` in the response goes back as `next_data_token` until it comes back empty.

`hyperbilling_get_recent_billing` replaces step 3 when one of its fixed views fits.

## hyperbilling_search_billing

| Argument | Required | Description |
| --- | --- | --- |
| `linked_accounts` | yes | 12-digit AWS linked account IDs |
| `granularity` | yes | `MONTHLY` or `DAILY` |
| `start`, `end` | yes | First and last day of the period, `YYYY-MM-DD` |
| `group_by` | yes | 1 to 5 grouping dimensions |
| `result` | yes | Measures to return per group |
| `regions` | no | Keep only these regions, as AWS usage report abbreviations (`APN2` for Seoul) |
| `service_codes` | no | Keep only these AWS service codes (`AmazonEC2`) |
| `next_data_token` | no | Token of the next page |
| `filter` | no | Extra `Filter` fields passed to the API as written |

Grouping dimensions: `USAGE_DATE`, `LINKED_ACCOUNT`, `SERVICE_CODE`, `REGION`, `PRODUCT_CATEGORY`, `USAGE_TYPE`, `OPERATION`, `INSTANCE_TYPE`, `RESOURCE_ID`, `LINEITEM_TYPE`, `DESCRIPTION`, `UNBLENDED_RATE`, `BILLING_PERIOD_START_DATE`, `RESERVED_PURCHASE_TYPE`, `RESERVED_PERIOD`, `RESERVED_INSTANCE_ID`, `RESERVED_ACCOUNT`, `SAVINGS_PLANS_ARN`, `SAVINGS_PLANS_ACCOUNT`, `SAVINGS_PLANS_TYPE`, `SAVINGS_PLANS_PERIOD`, `SAVINGS_PLANS_PURCHASE_OPTION`, and `TAG:<key>` with a key exactly as `hyperbilling_list_tag_keys` returns it (`TAG:user:Team`).

Measures: `USAGE_COST`, `USAGE_QUANTITY`, `SP_EFFECTIVE_COST`, `SP_TOT_COMMITMENT_TO_DATE`, `SP_USED_COMMITMENT`.

The API silently ignores a `filter` key it does not know. An unfiltered result means the key was wrong.

## hyperbilling_get_recent_billing

| View | Report |
| --- | --- |
| `latest` | Current month total so far |
| `last-3-months` | Monthly totals of the last three months, current month included |
| `last-6-months` | Monthly totals of the last six months, current month included |
| `latest-month-daily` | Current month by day |
| `latest-from-last-month-daily` | The month ending at the newest billing date, by day |
| `latest-by-region` | Current month by region |
| `latest-by-product` | Current month by AWS service code |

## Limits

These are HyperBilling limits, passed through unchanged.

| Limit | Value |
| --- | --- |
| Requests per hour | 500 |
| Rows per response | 1000 |
| Grouping dimensions | 5 |
| Daily period | 31 days |
| Monthly period | 24 months |

## Errors

A request HyperBilling rejects comes back as a tool error that carries the API message, so the agent can correct the call. Only a request that got no response fails the MCP call itself.

| Status | Meaning |
| --- | --- |
| 204 | No billing data matches the query. Returned as a normal, empty result |
| 400 | A required field is missing, a value is malformed, or a limit was exceeded |
| 401 | A header is missing, the key expired, or the client ID does not own the key |
| 403 | Authenticated, but no linked account is visible to this client ID |
| 429 | The hourly request limit was exceeded |

## Argument coercion

Some MCP clients send every argument as a string. An array argument accepts a JSON-encoded array or a comma-separated string, and an object argument accepts a JSON-encoded object.

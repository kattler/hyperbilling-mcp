use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Map, Value, json};

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct NoArgs {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkedAccountsArgs {
    #[schemars(
        description = "12-digit AWS linked account IDs to query (from hyperbilling_list_linked_accounts)"
    )]
    pub linked_accounts: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[schemars(inline)]
pub enum Granularity {
    #[serde(rename = "MONTHLY", alias = "monthly")]
    Monthly,
    #[serde(rename = "DAILY", alias = "daily")]
    Daily,
}

impl Granularity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Monthly => "MONTHLY",
            Self::Daily => "DAILY",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchBillingArgs {
    #[schemars(
        description = "12-digit AWS linked account IDs to query (from hyperbilling_list_linked_accounts)"
    )]
    pub linked_accounts: Vec<String>,
    #[schemars(description = "MONTHLY (period up to 24 months) or DAILY (period up to 31 days)")]
    pub granularity: Granularity,
    #[schemars(description = "first day of the period, YYYY-MM-DD")]
    pub start: String,
    #[schemars(
        description = "last day of the period, YYYY-MM-DD, not later than the current month"
    )]
    pub end: String,
    #[schemars(
        description = "1 to 5 grouping dimensions: USAGE_DATE, LINKED_ACCOUNT, SERVICE_CODE, REGION, PRODUCT_CATEGORY, USAGE_TYPE, OPERATION, INSTANCE_TYPE, RESOURCE_ID, LINEITEM_TYPE, DESCRIPTION, UNBLENDED_RATE, BILLING_PERIOD_START_DATE, RESERVED_PURCHASE_TYPE, RESERVED_PERIOD, RESERVED_INSTANCE_ID, RESERVED_ACCOUNT, SAVINGS_PLANS_ARN, SAVINGS_PLANS_ACCOUNT, SAVINGS_PLANS_TYPE, SAVINGS_PLANS_PERIOD, SAVINGS_PLANS_PURCHASE_OPTION, or TAG:<key> with a key exactly as hyperbilling_list_tag_keys returns it (TAG:user:Team, TAG:aws:eks:namespace). A row without a value for a dimension returns null for it"
    )]
    pub group_by: Vec<String>,
    #[schemars(
        description = "measures to return per group: USAGE_COST, USAGE_QUANTITY, SP_EFFECTIVE_COST, SP_TOT_COMMITMENT_TO_DATE, SP_USED_COMMITMENT"
    )]
    pub result: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these regions, as AWS usage report abbreviations (APN2 for Seoul, USE1 for N. Virginia). Omit for every region"
    )]
    pub regions: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these AWS service codes as SERVICE_CODE returns them (AmazonEC2, AmazonRDS). Omit for every service"
    )]
    pub service_codes: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only rows that carry this tag key, exactly as hyperbilling_list_tag_keys returns it (user:Team). Omit for no tag filter"
    )]
    pub tag_key: String,
    #[serde(default)]
    #[schemars(
        description = "with tag_key, keep only rows whose tag has one of these values. Omit to match any value of the key"
    )]
    pub tag_values: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these operations as OPERATION returns them (RunInstances, PutObject). Omit for every operation"
    )]
    pub operations: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these usage types as USAGE_TYPE returns them (APN2-TimedStorage-ByteHrs). Omit for every usage type"
    )]
    pub usage_types: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these line item types as LINEITEM_TYPE returns them (Usage, Discount, Support). Omit for every type"
    )]
    pub line_item_types: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "keep only these product categories as PRODUCT_CATEGORY returns them. Omit for every category"
    )]
    pub product_categories: Vec<String>,
    #[serde(default)]
    #[schemars(
        description = "non-empty NextDataToken from the previous response, to fetch the next page of up to 1000 rows. Omit on the first call. An empty NextDataToken in a response means there are no more pages"
    )]
    pub next_data_token: String,
    #[serde(default)]
    #[schemars(
        with = "Map<String, Value>",
        description = "extra Filter fields passed to the API as written, for filters this tool does not name: UseBillingPeriod (boolean), and the string arrays AZ, ResourceId, InstanceType, Description, ReservationAccount, ReservationPurchaseOption, ReservationPurchasePeriod, SavingsPlansArn, SavingsPlansTypes, SavingsPlansPurchasePeriod, SavingsPlansPurchaseOption, SavingsPlansAccount. The API silently ignores a key it does not know, so an unfiltered result means the key was wrong"
    )]
    pub filter: Option<Map<String, Value>>,
}

impl SearchBillingArgs {
    pub fn into_body(self) -> Value {
        let mut filter = self.filter.unwrap_or_default();
        filter.insert("LinkedAccount".to_string(), json!(self.linked_accounts));
        filter.insert("Granularity".to_string(), json!(self.granularity.as_str()));
        filter.insert(
            "TimePeriod".to_string(),
            json!({"Start": self.start, "End": self.end}),
        );
        if !self.regions.is_empty() {
            filter.insert("Region".to_string(), json!(self.regions));
        }
        for (key, values) in [
            ("ServiceCode", self.service_codes),
            ("Operation", self.operations),
            ("UsageType", self.usage_types),
            ("LineItemType", self.line_item_types),
            ("ProductCategory", self.product_categories),
        ] {
            if !values.is_empty() {
                filter.insert(key.to_string(), json!(values));
            }
        }
        if !self.tag_key.is_empty() {
            let mut tag = Map::new();
            tag.insert("Key".to_string(), json!(self.tag_key));
            if !self.tag_values.is_empty() {
                tag.insert("Values".to_string(), json!(self.tag_values));
            }
            filter.insert("Tag".to_string(), Value::Object(tag));
        }
        if !self.next_data_token.is_empty() {
            filter.insert("NextDataToken".to_string(), json!(self.next_data_token));
        }
        json!({"Filter": filter, "GroupBy": self.group_by, "Result": self.result})
    }
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[schemars(inline)]
#[serde(rename_all = "kebab-case")]
pub enum RecentView {
    Latest,
    #[serde(rename = "last-3-months")]
    Last3Months,
    #[serde(rename = "last-6-months")]
    Last6Months,
    LatestMonthDaily,
    LatestFromLastMonthDaily,
    LatestByRegion,
    LatestByProduct,
}

impl RecentView {
    pub const ALL: [Self; 7] = [
        Self::Latest,
        Self::Last3Months,
        Self::Last6Months,
        Self::LatestMonthDaily,
        Self::LatestFromLastMonthDaily,
        Self::LatestByRegion,
        Self::LatestByProduct,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Latest => "latest",
            Self::Last3Months => "last-3-months",
            Self::Last6Months => "last-6-months",
            Self::LatestMonthDaily => "latest-month-daily",
            Self::LatestFromLastMonthDaily => "latest-from-last-month-daily",
            Self::LatestByRegion => "latest-by-region",
            Self::LatestByProduct => "latest-by-product",
        }
    }

    pub const fn path(self) -> &'static str {
        match self {
            Self::Latest => "/v2/search/billing/latest",
            Self::Last3Months => "/v2/search/billing/last-3-months",
            Self::Last6Months => "/v2/search/billing/last-6-months",
            Self::LatestMonthDaily => "/v2/search/billing/latest-month-daily",
            // The upstream path is spelled "lastest".
            Self::LatestFromLastMonthDaily => "/v2/search/billing/lastest-from-last-month-daily",
            Self::LatestByRegion => "/v2/search/billing/latest/region",
            Self::LatestByProduct => "/v2/search/billing/latest/product",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecentBillingArgs {
    #[schemars(
        description = "12-digit AWS linked account IDs to query (from hyperbilling_list_linked_accounts)"
    )]
    pub linked_accounts: Vec<String>,
    #[schemars(
        description = "fixed report: latest (current month total so far), last-3-months and last-6-months (monthly totals including the current month), latest-month-daily (current month by day), latest-from-last-month-daily (the month ending at the newest billing date, by day), latest-by-region and latest-by-product (current month by region or AWS service code)"
    )]
    pub view: RecentView,
}

impl LinkedAccountsArgs {
    /// `/v1/search/tag` takes `LinkedAccount` at the top level, not under `Filter`.
    pub fn into_tag_body(self) -> Value {
        json!({"LinkedAccount": self.linked_accounts})
    }

    pub fn into_filter_body(self) -> Value {
        json!({"Filter": {"LinkedAccount": self.linked_accounts}})
    }
}

impl RecentBillingArgs {
    pub fn body(&self) -> Value {
        json!({"Filter": {"LinkedAccount": self.linked_accounts}})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn billing_body_merges_extra_filter_and_token() {
        let args: SearchBillingArgs = serde_json::from_value(json!({
            "linked_accounts": ["123456789012"],
            "granularity": "monthly",
            "start": "2026-09-01",
            "end": "2026-09-30",
            "group_by": ["SERVICE_CODE"],
            "result": ["USAGE_COST"],
            "next_data_token": "page-2",
            "regions": ["APN2"],
            "service_codes": ["AmazonEC2"],
            "tag_key": "user:Team",
            "tag_values": ["platform"],
            "operations": ["RunInstances"],
            "usage_types": ["APN2-BoxUsage:m7g.large"],
            "line_item_types": ["Usage"],
            "product_categories": ["Compute Instance"],
            "filter": {"Extra": ["x"], "Granularity": "ignored"}
        }))
        .unwrap();

        assert_eq!(
            args.into_body(),
            json!({
                "Filter": {
                    "LinkedAccount": ["123456789012"],
                    "Granularity": "MONTHLY",
                    "TimePeriod": {"Start": "2026-09-01", "End": "2026-09-30"},
                    "NextDataToken": "page-2",
                    "Region": ["APN2"],
                    "ServiceCode": ["AmazonEC2"],
                    "Operation": ["RunInstances"],
                    "UsageType": ["APN2-BoxUsage:m7g.large"],
                    "LineItemType": ["Usage"],
                    "ProductCategory": ["Compute Instance"],
                    "Tag": {"Key": "user:Team", "Values": ["platform"]},
                    "Extra": ["x"]
                },
                "GroupBy": ["SERVICE_CODE"],
                "Result": ["USAGE_COST"]
            })
        );
    }

    #[test]
    fn billing_body_omits_empty_token() {
        let args: SearchBillingArgs = serde_json::from_value(json!({
            "linked_accounts": ["123456789012"],
            "granularity": "DAILY",
            "start": "2026-09-01",
            "end": "2026-09-30",
            "group_by": ["USAGE_DATE"],
            "result": ["USAGE_COST"],
            "tag_values": ["ignored-without-a-key"]
        }))
        .unwrap();
        let body = args.into_body();
        assert_eq!(body["Filter"]["Granularity"], "DAILY");
        for key in [
            "NextDataToken",
            "Region",
            "ServiceCode",
            "Operation",
            "UsageType",
            "LineItemType",
            "ProductCategory",
            "Tag",
        ] {
            assert!(body["Filter"].get(key).is_none(), "{key} must be omitted");
        }
    }

    #[test]
    fn tag_key_alone_matches_any_value() {
        let args: SearchBillingArgs = serde_json::from_value(json!({
            "linked_accounts": ["123456789012"],
            "granularity": "MONTHLY",
            "start": "2026-09-01",
            "end": "2026-09-30",
            "group_by": ["TAG:user:Team"],
            "result": ["USAGE_COST"],
            "tag_key": "user:Team"
        }))
        .unwrap();
        assert_eq!(
            args.into_body()["Filter"]["Tag"],
            json!({"Key": "user:Team"})
        );
    }

    #[test]
    fn recent_views_map_to_fixed_paths() {
        let view = |name: &str| {
            serde_json::from_value::<RecentView>(json!(name))
                .unwrap()
                .path()
        };
        assert_eq!(view("latest"), "/v2/search/billing/latest");
        assert_eq!(view("last-3-months"), "/v2/search/billing/last-3-months");
        assert_eq!(view("last-6-months"), "/v2/search/billing/last-6-months");
        assert_eq!(
            view("latest-month-daily"),
            "/v2/search/billing/latest-month-daily"
        );
        assert_eq!(
            view("latest-from-last-month-daily"),
            "/v2/search/billing/lastest-from-last-month-daily"
        );
        assert_eq!(view("latest-by-region"), "/v2/search/billing/latest/region");
        assert_eq!(
            view("latest-by-product"),
            "/v2/search/billing/latest/product"
        );
        assert!(serde_json::from_value::<RecentView>(json!("../x")).is_err());
        for listed in RecentView::ALL {
            assert_eq!(view(listed.name()), listed.path());
        }
    }
}

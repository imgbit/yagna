/*
 * Yagna Market API
 *
 * ## Yagna Market The Yagna Market is a core component of the Yagna Network, which enables computational Offersand Demands circulation. The Market is open for all entities willing to buy computations (Demands) or monetize computational resources (Offers). ## Yagna Market API The Yagna Market API is the entry to the Yagna Market through which Requestors and Providers can publish their Demands and Offers respectively, find matching counterparty, conduct negotiations and make an agreement.  This version of Market API conforms with capability level 1 of the <a href=\"https://docs.google.com/document/d/1Zny_vfgWV-hcsKS7P-Kdr3Fb0dwfl-6T_cYKVQ9mkNg\"> Market API specification</a>.  Each of the two roles: Requestors and Providers have their own interface in the Market API.
 *
 * The version of the OpenAPI document: 1.2.0
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "properties")]
    pub properties: serde_json::Value,
    #[serde(rename = "constraints")]
    pub constraints: String,
    #[serde(rename = "prevProposalId", skip_serializing_if = "Option::is_none")]
    pub prev_proposal_id: Option<String>,
}

impl Proposal {
    pub fn new(id: String, properties: serde_json::Value, constraints: String) -> Proposal {
        Proposal {
            id,
            properties,
            constraints,
            prev_proposal_id: None,
        }
    }
}
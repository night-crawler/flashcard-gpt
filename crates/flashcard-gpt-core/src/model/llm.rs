use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct GptCardGroup {
    pub importance: u8,
    pub difficulty: u8,
    pub title: Arc<str>,
    pub tags: Vec<Arc<str>>,
    pub data: Option<Value>,
    pub cards: Vec<GptCard>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GptCard {
    pub title: Arc<str>,
    pub front: Arc<str>,
    pub back: Arc<str>,
    pub hints: Vec<Arc<str>>,
    pub difficulty: u8,
    pub importance: u8,
    pub tags: Vec<Arc<str>>,
}

impl GptCardGroup {
    pub fn from_gpt_response(input: &str) -> Result<Self, serde_json::Error> {
        let input = input.strip_prefix("```json").unwrap_or(input);
        let input = input.strip_prefix("```").unwrap_or(input);
        let input = input.strip_suffix("```").unwrap_or(input);

        serde_json::from_str(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use testresult::TestResult;

    #[test]
    fn test_parse() -> TestResult {
        let input = "```json\n{\n    \"importance\": 8,  \n    \"difficulty\": 6,  \n    \"title\": \"Largest Number (LeetCode)\", \n    \"tags\": [\"Medium\", \"Sorting\", \"Custom Sort\", \"String Manipulation\", \"Edge Cases\"], \n    \"data\": {\n        \"leetcode_link\": \"https://leetcode.com/problems/largest-number/\"\n    },\n    \"cards\": [\n        {\n            \"title\": \"Why Compare Numbers as Strings in Largest Number Problem?\", \n            \"front\": \"Why is it necessary to compare numbers as strings in the 'Largest Number' problem instead of comparing them as integers?\", \n            \"back\": \"Comparing numbers as strings allows us to evaluate the result of concatenating numbers in different orders. This ensures that the concatenation yielding the largest combined number is prioritized. For example, comparing 30 and 3 as integers would place 30 before 3, but concatenating them as strings ('330' vs. '303') reveals '330' is larger, so 3 should come first.\", \n            \"hints\": [\n                \"What happens if you compare 30 and 3 as integers?\", \n                \"What is the goal of concatenating numbers in different orders?\", \n                \"Consider two methods of concatenation: ab and ba.\"\n            ], \n            \"difficulty\": 6,  \n            \"importance\": 8,  \n            \"tags\": [\"String Manipulation\", \"Sorting\", \"Custom Comparator\"]\n        },\n        {\n            \"title\": \"Custom Sorting Logic for Largest Number\", \n            \"front\": \"Describe the custom sorting logic used to solve the 'Largest Number' problem.\", \n            \"back\": \"The custom sorting logic involves comparing two numbers as strings by concatenating them in both possible orders (e.g., ab and ba). The sorting decision is based on which concatenation produces a larger result. For example, for numbers a = '3' and b = '30', '330' > '303', so '3' should come before '30'.\", \n            \"hints\": [\n                \"What does ab and ba represent in the context of this logic?\",\n                \"Think about string comparison, not the absolute integer values.\"\n            ], \n            \"difficulty\": 7,  \n            \"importance\": 8,  \n            \"tags\": [\"Custom Sort\", \"String Comparison\", \"Greedy Strategy\"]\n        },\n        {\n            \"title\": \"Edge Case for Arrays of Zeros in Largest Number\", \n            \"front\": \"How do you handle the edge case where the input array contains multiple zeros (e.g., [0, 0, 0])? Why is this necessary?\", \n            \"back\": \"If the first number in the sorted list is '0', all other numbers must also be zeros. In this case, we return '0' instead of '000'. This is necessary to avoid leading zeros in the final result, which should express the zero concisely as just '0'.\", \n            \"hints\": [\n                \"What happens when all numbers in the input are zeros?\",\n                \"Consider what the final result should look like when the array has zeros.\"\n            ], \n            \"difficulty\": 4,  \n            \"importance\": 6,  \n            \"tags\": [\"Edge Cases\", \"Array Input\", \"String Manipulation\"]\n        }\n    ]\n}\n```";
        let card_group = GptCardGroup::from_gpt_response(input)?;

        assert!(card_group.data.is_some());
        assert_eq!(card_group.cards.len(), 3);

        Ok(())
    }
}

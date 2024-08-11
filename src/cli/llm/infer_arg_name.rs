use std::collections::HashMap;

use genai::chat::{ChatMessage, ChatRequest, ChatResponse};
use serde::{Deserialize, Serialize};

use super::{Error, Result, Wizard};
use crate::core::config::Config;

const MODEL: &str = "llama3-8b-8192";

#[derive(Default)]
pub struct InferArgsName {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Answer {
    suggestions: Vec<String>,
}

impl TryFrom<ChatResponse> for Answer {
    type Error = Error;

    fn try_from(response: ChatResponse) -> Result<Self> {
        let message_content = response.content.ok_or(Error::EmptyResponse)?;
        let text_content = message_content.text_as_str().ok_or(Error::EmptyResponse)?;
        Ok(serde_json::from_str(text_content)?)
    }
}

#[derive(Clone, Serialize)]
struct Question {
    arg: (String, String),
}

impl TryInto<ChatRequest> for Question {
    type Error = Error;

    fn try_into(self) -> Result<ChatRequest> {
        let content = serde_json::to_string(&self)?;
        let input = serde_json::to_string_pretty(&Question {
            arg: ("p1".to_string(), "Int".to_string()),
        })?;

        let output = serde_json::to_string_pretty(&Answer {
            suggestions: vec![
                "id".into(),
                "userId".into(),
                "count".into(),
            ],
        })?;

        Ok(ChatRequest::new(vec![
            ChatMessage::system(
                "Given the sample schema of a GraphQL field args suggest 5 meaningful names for the args.",
            ),
            ChatMessage::system("The name should be concise and preferably a single word"),
            ChatMessage::system("Example Input:"),
            ChatMessage::system(input),
            ChatMessage::system("Example Output:"),
            ChatMessage::system(output),
            ChatMessage::system("Ensure the output is in valid JSON format".to_string()),
            ChatMessage::system(
                "Do not add any additional text before or after the json".to_string(),
            ),
            ChatMessage::user(content),
        ]))
    }
}

impl InferArgsName {
    pub async fn generate(&mut self, config: &Config) -> Result<HashMap<String, String>> {
        let wizard: Wizard<Question, Answer> = Wizard::new(MODEL.to_string());

        let mut new_name_mappings: HashMap<String, String> = HashMap::new();

        // removed root type from types.
        let args_to_be_processed = config
            .types
            .iter()
            .filter(|(type_name, _)| !config.is_root_operation_type(type_name))
            .map(|(_, ty)| ty.fields.values().map(|v| v.args.to_owned().into_iter().collect::<Vec<_>>()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        let total = args_to_be_processed.len();
        for (i, (arg_name, arg)) in args_to_be_processed.into_iter().enumerate() {
            // convert type to sdl format.
            let question = Question {
                arg: (arg_name.to_owned(), arg.type_of.clone()),
            };

            let mut delay = 3;
            loop {
                let answer = wizard.ask(question.clone()).await;
                match answer {
                    Ok(answer) => {
                        let name = &answer.suggestions.join(", ");
                        for name in answer.suggestions {
                            if config.types.contains_key(&name)
                                || new_name_mappings.contains_key(&name)
                            {
                                continue;
                            }
                            new_name_mappings.insert(name, arg_name.to_owned());
                            break;
                        }
                        tracing::info!(
                            "Suggestions for {}: [{}] - {}/{}",
                            arg_name,
                            name,
                            i + 1,
                            total
                        );

                        // TODO: case where suggested names are already used, then extend the base
                        // question with `suggest different names, we have already used following
                        // names: [names list]`
                        break;
                    }
                    Err(e) => {
                        // TODO: log errors after certain number of retries.
                        if let Error::GenAI(_) = e {
                            // TODO: retry only when it's required.
                            tracing::warn!(
                                "Unable to retrieve a name for the arg '{}'. Retrying in {}s",
                                arg_name,
                                delay
                            );
                            tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                            delay *= std::cmp::min(delay * 2, 60);
                        }
                    }
                }
            }
        }

        Ok(new_name_mappings.into_iter().map(|(k, v)| (v, k)).collect())
    }
}

#[cfg(test)]
mod test {
    use genai::chat::{ChatRequest, ChatResponse, MessageContent};

    use super::{Answer, Question};

    #[test]
    fn test_to_chat_request_conversion() {
        let question = Question {
            arg: ("id".to_string(), "String".to_string()),
        };
        let request: ChatRequest = question.try_into().unwrap();
        insta::assert_debug_snapshot!(request);
    }

    #[test]
    fn test_chat_response_parse() {
        let resp = ChatResponse {
            content: Some(MessageContent::Text(
                "{\"suggestions\":[\"Post\",\"Story\",\"Article\",\"Event\",\"Brief\"]}".to_owned(),
            )),
            ..Default::default()
        };
        let answer = Answer::try_from(resp).unwrap();
        insta::assert_debug_snapshot!(answer);
    }
}

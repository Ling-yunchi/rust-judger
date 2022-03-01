use reqwest::header::HeaderMap;

use crate::engine::JudgeResult;

async fn post<'a>(url: &str, data: String) -> Result<String, Box<dyn std::error::Error>> {
    println!("{}", data);
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let res = client.post(url).headers(headers).body(data).send().await?;
    let res = res.text().await?;
    println!("{:?}", res);
    Ok(res)
}

const BASE_URL: &str = "http:///192.168.92.1:8000";
const RESULT_URL: &str = "/result";

pub async fn send_result(result: JudgeResult) {
    let data = serde_json::to_string(&result).unwrap();
    let url = BASE_URL.to_owned() + RESULT_URL;
    let res = post(&url, data).await.unwrap();
    if res == "OK" {
        println!("send result ok");
    } else {
        println!("send result failed");
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::JudgeCase;

    use super::*;

    #[tokio::test]
    async fn test_result() {
        let result = JudgeResult {
            id: "123".to_string(),
            result: JudgeCase::Accepted,
            case: 1,
            time: 100,
            memory: 100,
        };
        send_result(result).await;
    }

    #[tokio::test]
    async fn test_post() {
        let data = format!(
            "{{\
                \"id\":\"{}\",\
                \"case\":{},\
                \"result\":\"{}\",\
                \"time\":{},\
                \"memory\":{}\
            }}",
            "123".to_string(),
            1,
            "Accepted".to_string(),
            100,
            100
        );

        let url = BASE_URL.to_owned() + RESULT_URL;
        let res = post(&url, data).await.unwrap();
        assert_eq!(res, "OK");
    }
}

use crate::types;
use crate::Error;
use reqwest::header::{HeaderMap, HeaderValue};

/// Auth
/// [OAuth 인증 관련](https://apiportal.koreainvestment.com/apiservice/oauth2#L_214b9e22-8f2e-4fba-9688-587279f1061a)
pub struct Auth {
    client: reqwest::Client,
    endpoint_url: String,
    appkey: String,
    appsecret: String,
    hash: Option<String>,
    token: Option<String>,
    approval_key: Option<String>,
}

impl Auth {
    /// OAuth API 사용을 위한 구조체 생성
    /// hash, token, approval_key는 None으로 시작
    /// create_hash, create_token, create_approval_key 함수를 호출해야 hash, token 값이 저장됨
    pub fn new(
        client: &reqwest::Client,
        endpoint_url: &str,
        appkey: String,
        appsecret: String,
    ) -> Self {
        Self {
            client: client.clone(),
            endpoint_url: endpoint_url.to_string(),
            appkey,
            appsecret,
            hash: None,
            token: None,
            approval_key: None,
        }
    }

    /// 구조체에 저장되어 있는 hash를 반환
    pub fn get_hash(&self) -> Option<String> {
        self.hash.clone()
    }

    /// 구조체에 저장되어 있는 token을 반환
    pub fn get_token(&self) -> Option<String> {
        self.token.clone()
    }

    /// 구조체에 저장되어 있는 approval_key 반환
    pub fn get_approval_key(&self) -> Option<String> {
        self.approval_key.clone()
    }

    /// 실시간 (웹소켓) 접속키 발급[실시간-000]
    /// [Docs](https://apiportal.koreainvestment.com/apiservice/oauth2#L_5c87ba63-740a-4166-93ac-803510bb9c02)
    /// 웹소켓 접속키를 발급받아서 반환함과 동시에 구조체의 approval_key 업데이트
    pub async fn create_approval_key(&mut self) -> Result<String, Error> {
        let approval_key = self
            .client
            .post(format!("{}/oauth2/Approval", self.endpoint_url))
            .header("Content-Type", "application/json; utf8")
            .body(format!(
                "{{\"grant_type\": \"client_credentials\", \"appkey\": \"{}\", \"appsecret\":\"{}\"}}",
                self.appkey, self.appsecret
            ))
            .send()
            .await?
            .json::<types::ApprovalKeyCreationResponse>()
            .await?
            .get_approval_key();
        self.approval_key = Some(approval_key.clone());
        Ok(approval_key)
    }

    /// Hashkey
    /// [Docs](https://apiportal.koreainvestment.com/apiservice/oauth2#L_214b9e22-8f2e-4fba-9688-587279f1061a)
    /// hash값을 얻어와서 반환함과 동시에 구조체의 hash를 업데이트
    pub async fn create_hash(&mut self) -> Result<String, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert("appkey", HeaderValue::from_str(&self.appkey).unwrap());
        headers.insert("appsecret", HeaderValue::from_str(&self.appsecret).unwrap());
        let hash = self
            .client
            .post(format!("{}/uapi/hashkey", self.endpoint_url))
            .headers(headers)
            .body("{}")
            .send()
            .await?
            .json::<types::HashKeyResponse>()
            .await?
            .get_hash();
        self.hash = Some(hash.clone());
        Ok(hash)
    }

    /// 접근토큰발급(P)[인증-001]
    /// [Docs](https://apiportal.koreainvestment.com/apiservice/oauth2#L_fa778c98-f68d-451e-8fff-b1c6bfe5cd30)
    /// token값을 얻어와서 반환함과 동시에 구조체의 token을 업데이트
    pub async fn create_token(&mut self) -> Result<String, Error> {
        let token = self
            .client
            .post(format!("{}/uapi/tokenP", self.endpoint_url))
            .header("Content-Type", "application/json")
            .body(format!(
                "{{\"appkey\": \"{}\",\"appsecret\":\"{}\"}}",
                self.appkey, self.appsecret
            ))
            .send()
            .await?
            .json::<types::TokenCreationResponse>()
            .await?
            .get_access_token();
        self.token = Some(token.clone());
        Ok(token)
    }

    /// 접근토큰폐기(P)[인증-002]
    /// [Docs](https://apiportal.koreainvestment.com/apiservice/oauth2#L_dd3cb447-5034-4711-8c88-62c913429c7b)
    /// token을 폐기. 폐기 결과 리스폰스를 그대로 반환
    /// 리스폰스 형식: {
    ///     code: u32,
    ///     message: String,
    /// }
    pub async fn revoke_token(&self) -> Result<types::TokenRevokeResponse, Error> {
        Ok(self
            .client
            .post(format!("{}/uapi/revokeP", &self.endpoint_url))
            .header("Content-Type", "application/json")
            .body(format!(
                "{{\"appkey\":\"{}\",\"appsecret\":\"{}\",\"token\":\"{}\"}}",
                self.appkey,
                self.appsecret,
                self.token.clone().unwrap()
            ))
            .send()
            .await?
            .json::<types::TokenRevokeResponse>()
            .await?)
    }
}
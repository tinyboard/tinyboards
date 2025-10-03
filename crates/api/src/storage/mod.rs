use opendal::{Operator, services, layers::LoggingLayer, ErrorKind};
use tinyboards_utils::{TinyBoardsError, settings::structs::Settings};
use url::Url;

#[derive(Clone)]
pub struct StorageBackend {
    operator: Operator,
    backend_type: StorageType,
    base_url: String,
}

#[derive(Clone, Debug)]
pub enum StorageType {
    Filesystem,
    S3,
    Azure,
    Gcs,
}

impl StorageBackend {
    pub async fn from_settings(settings: &Settings) -> Result<Self, TinyBoardsError> {
        let backend_str = settings.storage.backend.as_deref().unwrap_or("fs");
        let (operator, backend_type) = match backend_str {
            "s3" => {
                let s3_config = settings.storage.s3.as_ref()
                    .ok_or_else(|| TinyBoardsError::from_message(500, "S3 config missing"))?;

                let mut s3_builder = services::S3::default()
                    .bucket(&s3_config.bucket)
                    .region(&s3_config.region)
                    .access_key_id(&s3_config.access_key_id)
                    .secret_access_key(&s3_config.secret_access_key);

                if let Some(endpoint) = &s3_config.endpoint {
                    s3_builder = s3_builder.endpoint(endpoint);
                }

                let op = Operator::new(s3_builder)
                    .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to initialize S3"))?
                    .layer(LoggingLayer::default())
                    .finish();

                (op, StorageType::S3)
            },
            "azure" => {
                let azure_config = settings.storage.azure.as_ref()
                    .ok_or_else(|| TinyBoardsError::from_message(500, "Azure config missing"))?;

                let op = Operator::new(
                    services::Azblob::default()
                        .container(&azure_config.container)
                        .account_name(&azure_config.account_name)
                        .account_key(&azure_config.account_key)
                )
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to initialize Azure"))?
                .layer(LoggingLayer::default())
                .finish();

                (op, StorageType::Azure)
            },
            "gcs" => {
                let gcs_config = settings.storage.gcs.as_ref()
                    .ok_or_else(|| TinyBoardsError::from_message(500, "GCS config missing"))?;

                let op = Operator::new(
                    services::Gcs::default()
                        .bucket(&gcs_config.bucket)
                        .credential(&gcs_config.credential)
                )
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to initialize GCS"))?
                .layer(LoggingLayer::default())
                .finish();

                (op, StorageType::Gcs)
            },
            "fs" | "filesystem" => {
                let fs_config = settings.storage.fs.as_ref()
                    .ok_or_else(|| TinyBoardsError::from_message(500, "FS config missing"))?;

                let op = Operator::new(
                    services::Fs::default()
                        .root(&fs_config.root)
                )
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to initialize filesystem"))?
                .layer(LoggingLayer::default())
                .finish();

                (op, StorageType::Filesystem)
            },
            _ => return Err(TinyBoardsError::from_message(
                500,
                &format!("Unsupported storage backend: {}", backend_str)
            )),
        };

        Ok(Self {
            operator,
            backend_type,
            base_url: settings.get_protocol_and_hostname(),
        })
    }

    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    pub fn backend_type(&self) -> &StorageType {
        &self.backend_type
    }

    pub async fn write(&self, key: &str, data: Vec<u8>) -> Result<(), TinyBoardsError> {
        self.operator.write(key, data).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to write file"))?;
        Ok(())
    }

    pub async fn write_streaming<R>(&self, key: &str, reader: R) -> Result<(), TinyBoardsError>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
    {
        use tokio::io::AsyncReadExt;

        let mut writer = self.operator.writer_with(key)
            .chunk(8 * 1024 * 1024)  // 8MB chunks
            .concurrent(4)
            .await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to create writer"))?;

        let mut reader = reader;
        let mut buffer = vec![0u8; 8 * 1024 * 1024];

        loop {
            let n = reader.read(&mut buffer).await
                .map_err(|e| TinyBoardsError::from_message(500, &format!("Read error: {}", e)))?;
            if n == 0 { break; }

            // Clone the buffer slice to create owned data
            let chunk = buffer[..n].to_vec();
            writer.write(chunk).await
                .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to write chunk"))?;
        }

        writer.close().await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to close writer"))?;

        Ok(())
    }

    pub async fn read(&self, key: &str) -> Result<Vec<u8>, TinyBoardsError> {
        let buffer = self.operator.read(key).await
            .map_err(|e| match e.kind() {
                ErrorKind::NotFound => TinyBoardsError::from_message(404, "File not found"),
                _ => TinyBoardsError::from_error_message(e, 500, "Failed to read file"),
            })?;
        Ok(buffer.to_vec())
    }

    pub async fn delete(&self, key: &str) -> Result<(), TinyBoardsError> {
        self.operator.delete(key).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to delete file"))
    }

    pub async fn exists(&self, key: &str) -> Result<bool, TinyBoardsError> {
        self.operator.exists(key).await
            .map_err(|e| TinyBoardsError::from_error_message(e, 500, "Failed to check existence"))
    }

    pub fn get_public_url(&self, key: &str) -> String {
        format!("{}/media/{}", self.base_url, key)
    }
}

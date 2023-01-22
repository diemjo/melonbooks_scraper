use webhook::client::{WebhookClient};

use crate::{model::Product, CONFIGURATION};
use crate::common::error::{Result, Error};

const IMAGE_URL: &str = "https://karpador.moe/images/favicon.png";
const DISCORD_URL: &str = "https://discord.com/api/webhooks/";

pub async fn notify_new_products<T: AsRef<Product>>(products: &[T], artist: &str) -> Result<()> {
    if CONFIGURATION.discord_api_key.is_none() {
        return Ok(());
    }
    if products.is_empty() {
        return Ok(());
    }
    let url = format!("{}{}", DISCORD_URL, CONFIGURATION.discord_api_key.as_ref().unwrap());
    notify_new_products_to(products, artist, &url).await?;
    Ok(())
}

async fn notify_new_products_to<T: AsRef<Product>>(products: &[T], artist: &str, url: &str) -> Result<()> {
    let client: WebhookClient = WebhookClient::new(&url);
    for product_chunk in products.chunks(5) {
        client.send(|mut message| {
            message = message
                .content(&format!("{}: new products available:", artist))
                .username("MelonbookScraper")
                .avatar_url(IMAGE_URL);
            for product in product_chunk {
                let product = product.as_ref();
                message = message
                    .embed(|embed| embed
                        .title(&product.title)
                        .description(&product.url)
                        .thumbnail(&product.img_url)
                    );
            }
            message
        }).await.or_else(|e| Err(Error::DiscordError(e.to_string())))?;
        tokio::time::sleep(core::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

pub async fn notify_product_reruns<T: AsRef<Product>>(products: &[T], artist: &str) -> Result<()> {
    if CONFIGURATION.discord_api_key.is_none() {
        return Ok(());
    }
    if products.is_empty() {
        return Ok(());
    }
    let url = format!("{}{}", DISCORD_URL, CONFIGURATION.discord_api_key.as_ref().unwrap());
    notify_product_reruns_to(products, artist, &url).await?;
    Ok(())
}

async fn notify_product_reruns_to<T: AsRef<Product>>(products: &[T], artist: &str, url: &str) -> Result<()> {
    let client: WebhookClient = WebhookClient::new(&url);
    for product_chunk in products.chunks(5) {
        client.send(|mut message| {
            message = message
            .content(&format!("{}: products available again", artist))
            .username("MelonbookScraper")
            .avatar_url(IMAGE_URL);
            for product in product_chunk {
                let product = product.as_ref();
                message = message
                    .embed(|embed| embed
                        .title(&product.title)
                        .description(&product.url)
                        //.author(&product.associated_artist, None, None)
                        .thumbnail(&product.img_url))
            }
            message
        }).await.or_else(|e| Err(Error::DiscordError(e.to_string())))?;
        tokio::time::sleep(core::time::Duration::from_secs(1)).await;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use chrono::{Utc};

    use crate::{model::Product};

    use super::{notify_new_products_to, DISCORD_URL, notify_product_reruns_to};

    #[tokio::test]
    async fn test_send_new() {
        let url = format!("{}{}", DISCORD_URL, "1058185039529123962/5rRXWB9WlI3shB58NwGQ4fafvMsqdCg24ZogaqX32YH7YSwdkWMa7-foHAjywwiUOit_");
        notify_new_products_to(&vec![ nana_prod1(), nana_prod2() ] as &Vec<Product>, "nana", &url).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_rerun() {
        let url = format!("{}{}", DISCORD_URL, "1058185039529123962/5rRXWB9WlI3shB58NwGQ4fafvMsqdCg24ZogaqX32YH7YSwdkWMa7-foHAjywwiUOit_");
        notify_product_reruns_to(&vec![ nana_prod1(), nana_prod2(), nana_prod1(), nana_prod2(), nana_prod1(), nana_prod2() ] as &Vec<Product>, "nana", &url).await.unwrap();
        notify_product_reruns_to(&vec![ kantoku_prod1() ] as &Vec<Product>, "カントク", &url).await.unwrap();
    }

    fn nana_prod1() -> Product {
        Product::new(
            "https://www.melonbooks.co.jp/detail/detail.php?product_id=1793662".to_string(),
            "アクリルキューブ nana Uribou New Yaer2023".to_string(),
            "nana".to_string(),
            vec!["nana".to_string()],
            "https://melonbooks.akamaized.net/user_data/packages/resize_image.php?image=217001225510.jpg".to_string(),
            Utc::now().date_naive(),
            crate::model::Availability::Available
        )
    }

    fn nana_prod2() -> Product {
        Product::new(
            "https://www.melonbooks.co.jp/detail/detail.php?product_id=1704677".to_string(),
            "【2次受注】A3キャラファイングラフ nana 冬の物語(一般差分)".to_string(),
            "nana".to_string(),
            vec!["nana".to_string()],
            "https://melonbooks.akamaized.net/user_data/packages/resize_image.php?image=217001211823.jpg".to_string(),
            Utc::now().date_naive(),
            crate::model::Availability::Available
        )
    }

    fn kantoku_prod1() -> Product {
        Product::new(
            "https://www.melonbooks.co.jp/detail/detail.php?product_id=1664591".to_string(),
            "【アクリルコースター】くるみ-JKくるみちゃんは甘やかしたい。-".to_string(),
            "カントク".to_string(),
            vec!["カントク".to_string()],
            "https://melonbooks.akamaized.net/user_data/packages/resize_image.php?image=215001104593.jpg".to_string(),
            Utc::now().date_naive(),
            crate::model::Availability::Preorder
        )
    }
}
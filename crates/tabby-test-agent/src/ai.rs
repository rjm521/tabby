use serde_json::json;

/// 调用本地tabby服务生成测试用例
pub async fn generate_test_case_with_rig(api_desc: &str) -> String {
    // 构造测试用例生成的prompt
    let prompt = format!(
        "请为如下接口生成详细的测试用例，包含正向和异常场景。请用中文回答，并按照以下格式输出：\n\n\
        1. 测试场景概述\n\
        2. 测试用例列表（每个用例包含：用例名称、前置条件、测试步骤、预期结果）\n\
        3. 测试数据准备\n\
        4. 注意事项\n\n\
        接口描述：{}",
        api_desc
    );

    // 调用本地tabby聊天服务
    let payload = json!({
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7,
        "max_tokens": 2000
    });

    // 发送HTTP请求到本地tabby服务
    let client = reqwest::Client::new();
    match client
        .post("http://localhost:8080/v1/chat/completions")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => {
                        // 提取生成的文本
                        if let Some(choices) = json["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                if let Some(message) = choice["message"].as_object() {
                                    if let Some(content) = message["content"].as_str() {
                                        return content.to_string();
                                    }
                                }
                            }
                        }
                        "无法解析响应格式".to_string()
                    }
                    Err(e) => format!("解析响应失败: {}", e),
                }
            } else {
                format!("服务返回错误状态: {}", response.status())
            }
        }
        Err(e) => {
            // 如果调用本地服务失败，返回一个模拟的测试用例
            format!(
                "## 测试用例（模拟生成）\n\n\
                **接口描述**: {}\n\n\
                ### 1. 测试场景概述\n\
                - 验证接口的基本功能\n\
                - 验证异常输入的处理\n\
                - 验证边界条件的处理\n\n\
                ### 2. 测试用例列表\n\n\
                **用例1: 正常请求测试**\n\
                - 前置条件: 服务正常运行\n\
                - 测试步骤: 发送有效的请求参数\n\
                - 预期结果: 返回200状态码和正确的响应数据\n\n\
                **用例2: 参数验证测试**\n\
                - 前置条件: 服务正常运行\n\
                - 测试步骤: 发送无效或缺失的参数\n\
                - 预期结果: 返回400状态码和错误信息\n\n\
                **用例3: 异常场景测试**\n\
                - 前置条件: 服务正常运行\n\
                - 测试步骤: 模拟各种异常情况\n\
                - 预期结果: 返回适当的错误状态码\n\n\
                ### 3. 测试数据准备\n\
                - 准备有效的测试数据\n\
                - 准备各种异常输入数据\n\
                - 准备边界值测试数据\n\n\
                ### 4. 注意事项\n\
                - 确保测试环境稳定\n\
                - 注意请求频率限制\n\
                - 关注响应时间和性能\n\n\
                (注: 本地服务连接失败，已生成模拟测试用例。错误: {})",
                api_desc, e
            )
        }
    }
}
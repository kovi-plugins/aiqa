# aiqa

aiqa 使用 feature 区分不同协议后端，需手动选择一个启用。

| Feature | 协议 | 适用版本 |
|---|---|---|
| `milky` | 任意 Milky 服务端 | kovi >= 0.13 |
| `napcat-onebot` | NapCat（OneBot V11） | kovi >= 0.13 |



aiqa 是一个一次性ai问答插件，用于在kovi中实现ai问答功能。

初次使用需要现在 data/kovi-plugin-aiqa/config.json 中配置。

配置好请重载插件，使用消息命令或者重启bot。

## 依赖

系统里需要安装 chrome ? 暂不清楚，试着来吧。

```
sudo apt update && sudo apt install -y chromium
```

顺便安装一下字体

```
sudo apt install fonts-noto
```

## 使用说明

默认配置下：

使用 `%` 符号，例如 `%你好，1+1等于几？`，调用ai问答并用图片返回结果。

使用 `%%` 双符号，例如 `%%你好，1+1等于几？`，使用文本返回结果。

> [!warning]
> 配置调用符号，请只使用一个字符，插件内规定这个配置的类型为 `char` 。
>

可使用任意适配openai格式的模型。


## 旧版说明

> ⚠️ `Lagrange.OneBot` 在 aiqa 已不再维护，如需使用请锁定 `kovi-plugin-aiqa = "0.1"`（旧版仍支持 `kovi-plugin-expand-lagrange`）。
> 
> 锁定此 Git 版本，请使用以下依赖配置：
> ```
> kovi-plugin-aiqa = { git = "https://github.com/kovi-plugins/aiqa", rev = "17fa1e49f5e86729181f0e3d0e2552be2435b32c" }
> 
> # # 如果Kovi >= 0.13 请加多以下配置强制指定锁定这两个拓展api的版本
> # [patch.crates-io]
> # kovi-plugin-expand-napcat = { version = "0.5" }
> # kovi-plugin-expand-lagrange = { version = "0.8" }
> ```
>
> 具体可见 [kovi-plugin-expand-lagrange](https://crates.io/crates/kovi-plugin-expand-lagrange/0.8.1) 的版本说明

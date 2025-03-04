# aiqa

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

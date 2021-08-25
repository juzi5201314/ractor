# ractor

> An actor model framework. Personal projects for research and learning.
> 
> 一个actor模型框架. 用于我个人学习与研究.

## TODO
* 实现分布式, 使用远程地址给actor发送消息.(就像akka)(进行中)
* 提供更方便的过程宏, 简化定义`actor`和`message handler`的过程.
* 使用hyper和tower实现一个基于ractor的http框架.(就像actix-web).
* 错误处理和恢复, 出现错误时优雅的停止或重置actor.
* 实现`父` `子`结构.(就像akka)
* 更多...如果你也感兴趣

---
> 非科班学生, 业余爱好者, 基础差, 如有实现错误欢迎指出.


## Remote 基础流程
![png](./assets/ractor-remote-base.png)

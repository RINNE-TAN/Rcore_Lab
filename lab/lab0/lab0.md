# Lab 0

虽然原文中没有提到lab 0，但配置环境的时候还是遇到些问题，先把配置环境遇到发一些小问题记录在这里

1. 在powershell输入wsl命令后不存在，但是在电脑配置项发现linux支持确实已经打开了。

   **解决方法**powershell管理员模式输入

   Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Windows-Subsystem-Linux

   原理不是很懂，明明应该和电脑配置项是一样的道理的），然后重启就好。

2. 进入wsl后在系统中输入一些命令会显示file read only或者bus error甚至not found的错误，种类比较多已经记不清了。

   **解决方法**后来发现其实是C盘内存满了，清理一下就好了）
   
3. 安装qemu编译依赖的包时失败，执行cargo install cargo-binutils时失败。

   **解决方法**第一个是因为网络原因，换了ubuntu的代理源就好了，第二个好像是因为一些依赖没安装好然后build失败？在我先安装完qemu的依赖包后再执行这个命令就成功了。

4. make run失败，显示qemu不存在

   **解决方法**教程中配置的qemu环境变量是临时的，终端重启后会失败，把这个环境变量写入配置文件持久化就好了。

5. make debug失败

   **解决方法**安装gdb



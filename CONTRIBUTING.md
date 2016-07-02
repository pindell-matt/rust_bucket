# Thank you for wanting to contribute!

# Please follow these guidelines for an issue or PR to be considered

* If there is no detailed description the PR or the Issue will not be considered
* If the issue is a feature request, fork the repo, and make a PR that does not break core functionality and ensures that all tests still pass
* If you write a new feature (database API), testing is required.
* If your language is offensive, forget about it!
* If what you submit in a PR does not work on Windows, OSX, or Linux, then you must make sure that it does!

### If your are on Windows/OSX/Linux: 

*Travis CI will check your commits on a PR.*

But to avoid failures on stable (**if you are unsure**) here is a Vagrantfile that you should setup!

**Please change the `<PATH_TO_YOUR_LOCAL_RUST_BUCKET_FORK_OR_PROJECT>` to your hosts project path!**

```
$script = <<SCRIPT
  sudo apt-get update
  sudo /usr/sbin/update-locale LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
  sudo curl -s https://static.rust-lang.org/rustup.sh | sh -s -- --channel=stable
  cd /home/vagrant/rust_bucket && cargo test
SCRIPT

Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"
  config.vm.provision :shell, :inline => $script
  config.vm.synced_folder "<PATH_TO_YOUR_LOCAL_RUST_BUCKET_FORK_OR_PROJECT>", "/home/vagrant/rust_bucket"
  config.vm.provider "virtualbox" do |vb|
    vb.memory = "1024"
  end
end
```

### Steps to follow when using Vagrant:

`vagrant up`

Now before pushing up a commit do the following:

`vagrant ssh`

`cd rust_bucket && cargo test`

If all the tests pass, go ahead and push to your branch origin.

Travis CI will let us know everything is good to go!

### This is a WIP and more requirements may be added in the future!
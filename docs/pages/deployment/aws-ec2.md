# Example: AWS EC2

This is an example configuration that shows how to deploy LiveCompositor to an AWS EC2 instance with Terraform configuration.

All examples are located in the [github.com/membraneframework-labs/live_compositor_deployment](https://github.com/membraneframework-labs/live_compositor_deployment) repository:
- `project` directory includes an example Membrane project that can consume multiple streams over RTMP and host the composed stream as an HLS playlist.
- `aws-ec2-terraform` directory includes an example Terraform+Packer configuration for building an AMI (Amazon Machine Image) and deploying it to EC2.

### Prerequisites

- Terraform
- Packer
- Elixir - required to build an example project
- FFmpeg - used to send/receive streams from/to the compositor

### CPU vs GPU rendering trade-off

- `GPU+CPU` - LiveCompositor uses `wgpu` (implementation of WebGPU standard written in Rust) for rendering. However, all decoding and
encoding still happens on the CPU. When running on GPU the rendering cost should be negligible compared to the decoding/encoding.
- `CPU-only` - When running on a CPU-only instance, all `WebGPU` code is emulated on the CPU. Unless your encoder quality is set very
high, rendering will use most of the CPU processing time.

Actual price-to-performance can vary, but in general, CPU+GPU instances make more sense for fast encoder presets and complex rendering
pipelines. However, CPU-only can be more optimal when using simple layouts and prioritizing quality over performance with slower preset.

### How to deploy

:::warning
The example configuration is using `us-east-1` region. If you want to use a different one make sure to change it both in Packer
and Terraform configuration. Specifically, if you use EC2 instances with GPU, you might only have them available in some regions.
:::

### `CPU-only`

Go to **aws-ec2-terraform/packer** directory and run
```bash
packer build membrane.pkr.hcl
```
to build an AMI image with an example Membrane project. At the end of the process, the terminal will print the AMI ID, that will
be needed in the next step (something like `ami-0e18e9d7b8c037ec2`).

> The other `pkr.hcl` file in this directory (**standalone.pkr.hcl**) includes configuration for deploying just a standalone LiveCompositor
instance, so you can also go that route, but the rest of this guide assumes you are using the provided Membrane project.

Open **aws-ec2-terraform/main.tf**, find `aws_instance.demo_instance` definition and update the `ami` field with the AMI ID from the previous step.

In **aws-ec2-terraform** directory run:
```bash
terraform apply
```

### `CPU+GPU`

Go to **aws-ec2-terraform/packer** directory and run
```bash
packer build -var "with-gpu=true" membrane.pkr.hcl
```
to build an AMI image with an example Membrane project. At the end of the process, the terminal will print the AMI ID that, will
be needed in the next step (something like `ami-0e18e9d7b8c037ec2`).

Open **aws-ec2-terraform/main.tf**, find `aws_instance.demo_instance` definition and update the `ami` field with the AMI ID from the previous step.

In **aws-ec2-terraform** directory run:
```bash
terraform apply -var="with-gpu=true"
```

:::note
Instances with GPU like `g4dn.xlarge` are not available by default on AWS. You will need to request a quota increase from the AWS team to use them.
:::

### How to use

After everything is deployed you can open your AWS dashboard and find the public IP of the newly deployed instance.

To test the service, run in separate terminals:

- To receive the output stream
  ```
  ffplay http://YOUR_INSTANCE_IP:9001/index.m3u8
  ```
- To send an example input stream
  ```
  ffmpeg -re -f lavfi -i testsrc
    -vf scale=1280:720 -vcodec libx264 \
    -profile:v baseline -preset fast -pix_fmt yuv420p \
    -f flv rtmp://YOUR_INSTANCE_IP:9000/app/stream_key
  ```
  - You can run this command multiple times with different paths instead of `app/stream_key` to connect multiple streams.

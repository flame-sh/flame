# Flame: A Distributed System for Intelligent Workload

[![license](https://img.shields.io/github/license/flame-sh/flame)](http://github.com/flame-sh/flame)
[![RepoSize](https://img.shields.io/github/repo-size/flame-sh/flame)](http://github.com/flame-sh/flame)
[![Release](https://img.shields.io/github/release/flame-sh/flame)](https://github.com/flame-sh/flame/releases)
[![CII Best Practices](https://bestpractices.coreinfrastructure.org/projects/7299/badge)](https://bestpractices.coreinfrastructure.org/projects/7299)

Flame is a distributed system for intelligent workloads; it provides a suite of mechanisms that are commonly required by many classes of intelligent workload, 
including AI/ML, HPC, BigData and so on. Flame builds upon a decade and a half of experience running a wide variety of high performance workloads
at scale using several systems and platforms, combined with best-of-breed ideas and practices from the open source community.

## Motivation

As more and more intelligent workload patten are adopted for the innovation, a common workload runtime is helpful to speed up 
those intelligent workloads by following aspects:  

* **Scale**: Compared to the application in a single node, the Flame will scale up the workload to multiple nodes as much as possible to speed up it, e.g. distributed training; and it makes sure the resources are shared fairly between multiple tenants.   
* **Data sharing**: Data is one of key factor for intelligent workload; the Flame will schedule not only the workload, but also the data. A distributed cached will be introduced in Flame, and it will schedule data & resources together to improve data sharing.  
* **Mix workloads**: Batch (e.g. MPI) and Elastic are two major pattern for intelligent workloads, the Flame will manage those two kind of workload together by migrating 'message passing' to 'data driven'.
* **Roundtrip/Throughput**: Usually, intelligent workload includes tens of thousands of short tasks; the Flame leverages the latest features (e.g. Future, CondVar) to improve roundtrip and throughput in a large scale environment.

## Overall Architecture

![flame-architecture](https://raw.githubusercontent.com/flame-sh/flame/refs/heads/main/docs/images/flame-architecture.jpg)

### Terminologies

**Session:** One `Session` represents a group of tasks of a job, the `Session Scheduler` will allocate resources to each session based on scheduling configurations, by asking for resource manager (e.g. Kubernetes) to launch executors.

**Task:** The task of `Session` which includes the major algorithm of the job by task's metadata and input/output info, e.g. volume path.

**Executor:** The Executor will handle the lifecycle management of Application/Service which is user's code to execute tasks. Usually, the applications are not reused between sessions, but the image maybe reused to avoid download.

**Shim:** The protocol implementation of Executor to manage application, e.g. gRPC, Restful, stdio and so on. 

**Cache:** As the data is the key factor of intelligent workload, a distributed cache will be introduced in Flame to schedule data & resources together for better performance.

### Functionality

The Flame will accept connection from user's client, and create `Session`s for the job; the client keeps submit tasks to the session until closing it, pre-defined replica is not necessary.
The `Session Scheduler` will allocate resources to each session based on scheduling configurations, by asking for resource manager (e.g. Kubernetes) to launch executor.
The Executor will connect back to Flame by `gRPC` to pull tasks from related `Session` to reuse executor. The executor will be released/deleted if no more tasks in related session.

The service will get the notification when it's bound or unbound to the related session, so it can take action accordingly, e.g. connecting to database; and then, the service can pull tasks from `Session`,
and reuse those data to speed up execution.

In the future, the `Session scheduler` will provide several features to improve the performance and usage, e.g. proportion, delay release, min/max and so on.

## Quick Start Guide

In this guidance, [minikube](https://minikube.sigs.k8s.io/docs/) and [skaffold](https://skaffold.dev/) are used to start a local kuberentes
with Flame. After installing minikube and skaffold, we can start a local Flame cluster by the following steps: 

```shell
$ minikube start --driver=docker
$ skaffold run
```

After the Flame clsuter was launched, use the following steps to login into `flame-console` pod which is a debug tool for
both developer and SRE.

```shell
$ CONSOLE_POD=`kubectl get pod -n flame-system | grep flame-console | cut  -d" " -f 1`
$ kubectl exec -it ${CONSOLE_POD} -n flame-system -- /bin/bash
```

And then, let's verify it with `flmping` in the pod. In addition, there are also more meaningful examples [here](example).

```
# flmping -t 10000
Session <4> was created in <0 ms>, start to run <10,000> tasks in the session:

[100%] =============================================   10000/10000

<10,000> tasks was completed in <1,603 ms>.
```

We can check sessions' status by `flmctl` as follow, it also includes several sub-commands, e.g. `view`.

```
# flmctl list
ID        State     App            Slots     Pending   Running   Succeed   Failed    Created
1         Closed    pi             1         0         0         100       0         07:33:58
2         Closed    flmexec        1         0         0         10        0         07:34:12
3         Closed    flmexec        1         0         0         10000     0         07:34:17
4         Closed    flmexec        1         0         0         10000     0         08:34:20
```

## Blogs

* [Estimating the value of Pi using Monte Carlo](docs/blogs/evaluating-pi-by-monte-carlo.md)
* [Estimating the value of Pi by Flame Python Client](docs/blogs/evaluating-pi-by-flame-python.md)

## Reference

* **API**: https://github.com/flame-sh/flame/blob/main/rpc/protos/flame.proto


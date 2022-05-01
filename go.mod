module xflops.cn/flame

go 1.18

require (
	github.com/google/uuid v1.1.1
	github.com/spf13/pflag v1.0.5
	go.uber.org/automaxprocs v1.4.0
	google.golang.org/grpc v1.38.0
	google.golang.org/protobuf v1.26.0
	k8s.io/apimachinery v0.19.15
	k8s.io/component-base v0.19.15
	k8s.io/klog v1.0.0
	k8s.io/klog/v2 v2.2.0
	volcano.sh/volcano v1.5.1
)

require (
	cloud.google.com/go v0.81.0 // indirect
	github.com/beorn7/perks v1.0.1 // indirect
	github.com/blang/semver v3.5.0+incompatible // indirect
	github.com/cespare/xxhash/v2 v2.1.1 // indirect
	github.com/coreos/go-semver v0.3.0 // indirect
	github.com/coreos/go-systemd v0.0.0-20190321100706-95778dfbb74e // indirect
	github.com/coreos/pkg v0.0.0-20180928190104-399ea9e2e55f // indirect
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/docker/distribution v2.7.1+incompatible // indirect
	github.com/emicklei/go-restful v2.9.5+incompatible // indirect
	github.com/fsnotify/fsnotify v1.4.9 // indirect
	github.com/go-logr/logr v0.2.0 // indirect
	github.com/gogo/protobuf v1.3.2 // indirect
	github.com/golang/groupcache v0.0.0-20200121045136-8c9f03a8e57e // indirect
	github.com/golang/protobuf v1.5.2 // indirect
	github.com/google/cadvisor v0.37.5 // indirect
	github.com/google/go-cmp v0.5.5 // indirect
	github.com/google/gofuzz v1.1.0 // indirect
	github.com/googleapis/gnostic v0.4.1 // indirect
	github.com/hashicorp/errwrap v1.0.0 // indirect
	github.com/hashicorp/go-multierror v1.0.0 // indirect
	github.com/hashicorp/golang-lru v0.5.1 // indirect
	github.com/imdario/mergo v0.3.5 // indirect
	github.com/json-iterator/go v1.1.11 // indirect
	github.com/matttproud/golang_protobuf_extensions v1.0.2-0.20181231171920-c182affec369 // indirect
	github.com/modern-go/concurrent v0.0.0-20180306012644-bacd9c7ef1dd // indirect
	github.com/modern-go/reflect2 v1.0.1 // indirect
	github.com/opencontainers/go-digest v1.0.0-rc1 // indirect
	github.com/prometheus/client_golang v1.7.1 // indirect
	github.com/prometheus/client_model v0.2.0 // indirect
	github.com/prometheus/common v0.10.0 // indirect
	github.com/prometheus/procfs v0.1.3 // indirect
	go.etcd.io/etcd v0.5.0-alpha.5.0.20200819165624-17cef6e3e9d5 // indirect
	go.uber.org/atomic v1.7.0 // indirect
	go.uber.org/multierr v1.6.0 // indirect
	go.uber.org/zap v1.17.0 // indirect
	golang.org/x/crypto v0.0.0-20200622213623-75b288015ac9 // indirect
	golang.org/x/net v0.0.0-20210428140749-89ef3d95e781 // indirect
	golang.org/x/oauth2 v0.0.0-20210402161424-2e8d93401602 // indirect
	golang.org/x/sys v0.0.0-20210510120138-977fb7262007 // indirect
	golang.org/x/text v0.3.6 // indirect
	golang.org/x/time v0.0.0-20191024005414-555d28b269f0 // indirect
	google.golang.org/appengine v1.6.7 // indirect
	google.golang.org/genproto v0.0.0-20210602131652-f16073e35f0c // indirect
	gopkg.in/inf.v0 v0.9.1 // indirect
	gopkg.in/yaml.v2 v2.4.0 // indirect
	k8s.io/api v0.19.15 // indirect
	k8s.io/apiserver v0.19.15 // indirect
	k8s.io/client-go v0.19.15 // indirect
	k8s.io/cloud-provider v0.19.15 // indirect
	k8s.io/csi-translation-lib v0.19.15 // indirect
	k8s.io/kube-openapi v0.0.0-20200805222855-6aeccd4b50c6 // indirect
	k8s.io/kube-scheduler v0.0.0 // indirect
	k8s.io/kubernetes v1.19.15 // indirect
	k8s.io/utils v0.0.0-20200729134348-d5654de09c73 // indirect
	sigs.k8s.io/structured-merge-diff/v4 v4.1.2 // indirect
	sigs.k8s.io/yaml v1.2.0 // indirect
	stathat.com/c/consistent v1.0.0 // indirect
	volcano.sh/apis v1.5.0-beta.0 // indirect
)

replace (
	google.golang.org/grpc => google.golang.org/grpc v1.29.1
	k8s.io/api => k8s.io/api v0.19.15
	k8s.io/apiextensions-apiserver => k8s.io/apiextensions-apiserver v0.19.15
	k8s.io/apimachinery => k8s.io/apimachinery v0.19.15
	k8s.io/apiserver => k8s.io/apiserver v0.19.15
	k8s.io/cli-runtime => k8s.io/cli-runtime v0.19.15
	k8s.io/client-go => k8s.io/client-go v0.19.15
	k8s.io/cloud-provider => k8s.io/cloud-provider v0.19.15
	k8s.io/cluster-bootstrap => k8s.io/cluster-bootstrap v0.19.15
	k8s.io/code-generator => k8s.io/code-generator v0.19.15
	k8s.io/component-base => k8s.io/component-base v0.19.15
	k8s.io/cri-api => k8s.io/cri-api v0.19.15
	k8s.io/csi-translation-lib => k8s.io/csi-translation-lib v0.19.15
	k8s.io/klog => k8s.io/klog v1.0.0
	k8s.io/kube-aggregator => k8s.io/kube-aggregator v0.19.15
	k8s.io/kube-controller-manager => k8s.io/kube-controller-manager v0.19.15
	k8s.io/kube-proxy => k8s.io/kube-proxy v0.19.15
	k8s.io/kube-scheduler => k8s.io/kube-scheduler v0.19.15
	k8s.io/kubectl => k8s.io/kubectl v0.19.15
	k8s.io/kubelet => k8s.io/kubelet v0.19.15
	k8s.io/legacy-cloud-providers => k8s.io/legacy-cloud-providers v0.19.15
	k8s.io/metrics => k8s.io/metrics v0.19.15
	k8s.io/node-api => k8s.io/node-api v0.19.15
	k8s.io/sample-apiserver => k8s.io/sample-apiserver v0.19.15
	k8s.io/sample-cli-plugin => k8s.io/sample-cli-plugin v0.19.15
	k8s.io/sample-controller => k8s.io/sample-controller v0.19.15
)

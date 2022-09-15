#!/bin/bash

function yellow() { printf "\x1b[38;5;227m%s\e[0m " "${@}"; printf "\n"; }
function warn() { printf "\x1b[38;5;208m%s\e[0m " "${@}"; printf "\n"; }
function green() { printf "\x1b[38;5;048m%s\e[0m " "${@}"; printf "\n"; }
function red() { printf "\x1b[38;5;196m%s\e[0m " "${@}"; printf "\n"; }

if [[ "${ENV_NAME}" == "" ]]; then
    export ENV_NAME="dev"
fi
if [[ "${CA_FILE}" == "" ]]; then
    export CA_FILE="./kubernetes/tls/ca.pem"
fi
if [[ "${CA_KEY_FILE}" == "" ]]; then
    export CA_KEY_FILE="./kubernetes/tls/ca-key.pem"
fi
if [[ "${TLS_CHAIN_FILE}" == "" ]]; then
    export TLS_CHAIN_FILE="./kubernetes/tls/server-cert-chain.pem"
fi
if [[ "${TLS_KEY_FILE}" == "" ]]; then
    export TLS_KEY_FILE="./kubernetes/tls/server-key.pem"
fi

function add_helm_repo() {
    is_a_repo=$(helm repo list | grep -c strimzi)
    if [[ "${is_a_repo}" -eq 0 ]]; then
        echo "add_helm_repo - adding strimzi repo"
        vc="helm repo add strimzi https://strimzi.io/charts/"
        eval "${vc}"
        lt="$?"
        if [[ "${lt}" -ne 0 ]]; then
            red "failed to add strimzi repo with: ${vc}"
            exit 1
        fi
        echo "add_helm_repo - updating helm repo"
        vc="helm repo update"
        eval "${vc}"
        lt="$?"
        if [[ "${lt}" -ne 0 ]]; then
            red "failed to update helm repos with: ${vc}"
            exit 1
        fi
    fi
}

function deploy_client_and_cluster_tls_assets() {
    yellow "deploying client and cluster tls assets into ${ENV_NAME}"
    kubectl delete secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca-cert" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${ENV_NAME}-cluster-ca-cert" --from-file=ca.crt="${CA_FILE}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create ${ENV_NAME} kafka cluster-ca secret for ca.crt - stopping"
        exit 1
    fi
    kubectl label secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca-cert" strimzi.io/kind=Kafka strimzi.io/cluster="${ENV_NAME}"
    kubectl annotate secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca-cert" strimzi.io/ca-cert-generation=0

    kubectl delete secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${ENV_NAME}-cluster-ca" --from-file=ca.key="${CA_KEY_FILE}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create ${ENV_NAME} kafka cluster-ca secret for ca.key - stopping"
        exit 1
    fi
    kubectl label secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca" strimzi.io/kind=Kafka strimzi.io/cluster="${ENV_NAME}"
    kubectl annotate secret -n "${ENV_NAME}" "${ENV_NAME}-cluster-ca" strimzi.io/ca-key-generation=0

    # load the client CA pem
    # https://github.com/scholzj/strimzi-custom-ca-test/blob/de7218414501084851e30aff21bf3ff58dad1a68/load.sh#L23
    kubectl delete secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca-cert" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${ENV_NAME}-clients-ca-cert" --from-file=ca.crt="${CA_FILE}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create ${ENV_NAME} kafka clients-ca-cert secret for ca.crt - stopping"
        exit 1
    fi
    kubectl label secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca-cert" strimzi.io/kind=Kafka strimzi.io/cluster="${ENV_NAME}"
    kubectl annotate secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca-cert" strimzi.io/ca-cert-generation=0

    # load the client CA key
    kubectl delete secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca" --ignore-not-found
    kubectl create secret -n "${ENV_NAME}" generic "${ENV_NAME}-clients-ca" --from-file=ca.key="${CA_KEY_FILE}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create ${ENV_NAME} kafka clients-ca secret for ca.key - stopping"
        exit 1
    fi
    kubectl label secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca" strimzi.io/kind=Kafka strimzi.io/cluster="${ENV_NAME}"
    kubectl annotate secret -n "${ENV_NAME}" "${ENV_NAME}-clients-ca" strimzi.io/ca-key-generation=0
}

function deploy_listener_tls_assets() {
    yellow "deploying listener tls assets"
    kubectl delete secret -n "${ENV_NAME}" tls-kafka-cluster-0-server --ignore-not-found
    vc="kubectl create secret generic tls-kafka-cluster-0-server -n ${ENV_NAME} --from-file=server-cert-chain.pem=${TLS_CHAIN_FILE} --from-file=tls.key=${TLS_KEY_FILE}"
    eval "${vc}"
    lt="$?"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to create ${ENV_NAME} listener tls assets with command: ${vc}"
        exit 1
    fi
}

function deploy_operator() {
    echo "deploy_operator - checking if already running"
    is_running=$(kubectl get po -n st  | grep -c strimzi-cluster-operator)
    if [[ "${is_running}" -eq 0 ]]; then
        echo "deploy_operator - install crds"
        for crd in ./kubernetes/strimzi-kafka-operator/crds/*.yaml; do
            vc="kubectl apply -f ${crd} > /dev/null 2>&1"
            eval "${vc}"
            lt="$?"
            if [[ "${lt}" -ne 0 ]]; then
                red "failed deploying crd: ${crd} with: ${vc}"
                exit 1
            fi
        done
        echo "deploy_operator - deploying strimzi operator into st namespace"
        # use this to set vpn env vars like HTTPS_PROXY, https_proxy, HTTP_PROXY, http_proxy
        # --set extraEnvs[0].name=HTTPS_PROXY,extraEnvs[0].value=vpn_proxy_here
        # --set extraEnvs[1].name=https_proxy,extraEnvs[1].value=vpn_proxy_here
        # --set extraEnvs[0].name=HTTP_PROXY,extraEnvs[0].value=vpn_proxy_here
        # --set extraEnvs[1].name=http_proxy,extraEnvs[1].value=vpn_proxy_here
        helm upgrade --install \
            -n st \
            --create-namespace \
            --set watchAnyNamespace=true \
            --set logLevel=INFO,fullReconciliationIntervalMs=20000 \
            st \
            strimzi/strimzi-kafka-operator
        lt="$?"
        if [[ "${lt}" -ne 0 ]]; then
            red "failed to install strimzi operator with: ${vc}"
            exit 1
        fi
        echo "deploy_operator - waiting for strimzi operator to enter the ready state"
        vc="kubectl wait -n st --for=condition=Ready pod -l name=strimzi-cluster-operator --timeout=240s"
        eval "${vc}"
        lt="$?"
        if [[ "${lt}" -ne 0 ]]; then
            red "failed waiting on the strimzi operator pod to enter the ready state with: ${vc}"
            exit 1
        fi
    else
        echo "deploy_operator - strimzi already running"
    fi
}

function deploy_cluster() {
    echo "deploy_cluster - deploying into ${ENV_NAME} namespace"
    cluster_file="./kubernetes/${ENV_NAME}.yaml"
    if [[ ! -e "${cluster_file}" ]]; then
        red "missing kafka cluster file: ${cluster_file}"
        exit 1
    fi
    vc="kubectl apply -n ${ENV_NAME} -f ./kubernetes/${ENV_NAME}.yaml"
    eval "${vc}"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed to deploy cluster into ${ENV_NAME} with: ${vc}"
        exit 1
    fi
    echo "waiting for ${ENV_NAME} cluster to start up"
    vc="kubectl wait kafka/${ENV_NAME} -n ${ENV_NAME} --for=condition=Ready --timeout=300s"
    eval "${vc}"
    if [[ "${lt}" -ne 0 ]]; then
        red "failed waiting for ${ENV_NAME} cluster to start in the ${ENV_NAME} with: ${vc}"
        exit 1
    fi
}

yellow "starting strimzi kafka deployment into the ${ENV_NAME} namespace"

add_helm_repo
deploy_client_and_cluster_tls_assets
deploy_listener_tls_assets
deploy_operator
deploy_cluster

green "done"

exit 0

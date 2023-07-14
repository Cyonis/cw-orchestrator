# Starship Setup

> See [starship docs]() for detailed setup instructions.

**Spin up**

```bash
kind create cluster --name starship

helm install -f starship.yaml tutorial starship/devnet --version 0.1.38

# Port forward
/port-forward.sh --config=starship.yaml
```

**Check status**

```bash
kubectl get pods
```

**Delete Infrastructure**

```bash
helm delete tutorial
pkill -f "port-forward"
kind delete clusters starship
```
# faucet

https://github.com/cosmos/cosmjs/tree/main/packages/faucet



# MiniBit Kubernetes Deployment

This directory contains Kubernetes manifests for deploying MiniBit and the Velocity proxy to a Kubernetes cluster.

## Architecture

The deployment consists of two main components:

1. **Velocity Proxy**: Acts as the entry point for players, handling authentication and routing players to different game servers.
2. **MiniBit Server**: Runs all the game servers (lobby, parkour, sumo, etc.) in a single pod, each on different ports.

## Prerequisites

- A Kubernetes cluster (version 1.19+)
- `kubectl` configured to access your cluster
- Docker images built and pushed to a registry:
  - `pythonsnake602/minibit:latest`
  - `pythonsnake602/minibit-velocity:latest`

## Building Docker Images

Before deploying to Kubernetes, you need to build and push the Docker images:

```bash
# Build MiniBit image
docker build -t pythonsnake602/minibit:latest .
docker push pythonsnake602/minibit:latest

# Build Velocity proxy image
cd velocity
docker build -t pythonsnake602/minibit-velocity:latest .
docker push pythonsnake602/minibit-velocity:latest
```

**Note**: Replace `pythonsnake602` with your own Docker registry/organization name, and update the image references in the deployment files accordingly.

## Configuration

### 1. Generate Forwarding Secret

Before deploying, you must generate a secure forwarding secret. This secret is used for secure communication between Velocity and MiniBit servers.

```bash
# Generate a random secret
openssl rand -base64 12
```

### 2. Update Secret

Edit `k8s/secret.yaml` and replace `CHANGE_ME_PLEASE` with the generated secret:

```yaml
stringData:
  FORWARDING_SECRET: "your-generated-secret-here"
  VELOCITY_FORWARDING_SECRET: "your-generated-secret-here"
```

### 3. Storage Configuration

The MiniBit server requires persistent storage for world data and configurations. Review and adjust `k8s/minibit-pvc.yaml` if needed:

- Modify the storage size (default: 5Gi)
- Uncomment and set `storageClassName` if your cluster requires a specific storage class

### 4. Service Type

By default, the Velocity proxy uses a `LoadBalancer` service type. Depending on your cluster setup, you may need to change this:

- **Cloud providers** (AWS, GCP, Azure): `LoadBalancer` works out of the box
- **On-premises/local clusters**: Consider using `NodePort` instead

To use NodePort, edit `k8s/velocity-service.yaml`:

```yaml
spec:
  type: NodePort
  ports:
  - port: 25565
    targetPort: 25565
    nodePort: 30565  # Optional: specify a port in the 30000-32767 range
    protocol: TCP
    name: minecraft
```

## Deployment Steps

### Option 1: Deploy with kubectl apply

Deploy all resources in order:

```bash
# 1. Create namespace
kubectl apply -f k8s/namespace.yaml

# 2. Create secret
kubectl apply -f k8s/secret.yaml

# 3. Create persistent volume claim
kubectl apply -f k8s/minibit-pvc.yaml

# 4. Create ConfigMaps
kubectl apply -f k8s/velocity-config.yaml

# 5. Initialize MiniBit configuration (optional but recommended)
kubectl apply -f k8s/minibit-init-job.yaml

# Wait for the init job to complete
kubectl wait --for=condition=complete --timeout=300s job/minibit-init -n minibit

# 6. Deploy applications
kubectl apply -f k8s/minibit-deployment.yaml
kubectl apply -f k8s/velocity-deployment.yaml

# 7. Create services
kubectl apply -f k8s/minibit-service.yaml
kubectl apply -f k8s/velocity-service.yaml
```

### Option 2: Deploy all at once

```bash
kubectl apply -f k8s/
```

**Note**: When using this method, the init job may fail initially if the PVC isn't ready. Simply re-run the command or manually apply the init job after the PVC is bound.

## Verification

Check the status of your deployment:

```bash
# Check all resources in the minibit namespace
kubectl get all -n minibit

# Check pod status
kubectl get pods -n minibit

# View logs from Velocity proxy
kubectl logs -n minibit -l app=velocity-proxy -f

# View logs from MiniBit server
kubectl logs -n minibit -l app=minibit -f

# Get the external IP for the Velocity service
kubectl get svc velocity-service -n minibit
```

## Connecting to the Server

Once deployed, players can connect using:

- **LoadBalancer**: Use the EXTERNAL-IP from `kubectl get svc velocity-service -n minibit`
- **NodePort**: Use any cluster node IP with the assigned NodePort (default range: 30000-32767)

```bash
# For LoadBalancer
EXTERNAL_IP=$(kubectl get svc velocity-service -n minibit -o jsonpath='{.status.loadBalancer.ingress[0].ip}')
echo "Connect to: $EXTERNAL_IP:25565"

# For NodePort
NODE_IP=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[?(@.type=="ExternalIP")].address}')
NODE_PORT=$(kubectl get svc velocity-service -n minibit -o jsonpath='{.spec.ports[0].nodePort}')
echo "Connect to: $NODE_IP:$NODE_PORT"
```

## Scaling

### Horizontal Scaling (Multiple Proxy Instances)

You can scale the Velocity proxy for high availability:

```bash
kubectl scale deployment velocity-proxy -n minibit --replicas=3
```

**Note**: The MiniBit server currently runs as a single instance. Horizontal scaling of game servers would require architectural changes.

### Vertical Scaling (Resource Adjustment)

Adjust resource requests and limits in the deployment files based on your needs:

```yaml
resources:
  requests:
    memory: "2Gi"
    cpu: "1000m"
  limits:
    memory: "8Gi"
    cpu: "4000m"
```

## Troubleshooting

### Pods not starting

```bash
# Check pod events
kubectl describe pod <pod-name> -n minibit

# Check pod logs
kubectl logs <pod-name> -n minibit
```

### Connection issues

1. Verify services are running:
   ```bash
   kubectl get svc -n minibit
   ```

2. Check if Velocity can reach MiniBit servers:
   ```bash
   kubectl exec -it deployment/velocity-proxy -n minibit -- nc -zv minibit-service 25566
   ```

3. Verify the forwarding secret matches in both deployments

### Storage issues

```bash
# Check PVC status
kubectl get pvc -n minibit

# Check PV status
kubectl get pv
```

## Cleanup

To remove all MiniBit resources from your cluster:

```bash
kubectl delete namespace minibit
```

Or individually:

```bash
kubectl delete -f k8s/
```

## Advanced Configuration

### Custom Velocity Configuration

To customize Velocity settings, edit `k8s/velocity-config.yaml` and update the ConfigMap. After making changes, restart the Velocity pods:

```bash
kubectl apply -f k8s/velocity-config.yaml
kubectl rollout restart deployment/velocity-proxy -n minibit
```

### Database Integration

If your MiniBit server requires database connectivity, you can add database connection settings via environment variables or a separate ConfigMap.

### Ingress

For production deployments, consider using an Ingress controller instead of LoadBalancer:

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: minibit-ingress
  namespace: minibit
  annotations:
    # Ingress controller specific annotations
spec:
  rules:
  - host: minecraft.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: velocity-service
            port:
              number: 25565
```

**Note**: Minecraft protocol may require special ingress controller configuration or TCP passthrough.

## Security Considerations

1. **Change the default secret**: Always use a strong, randomly generated forwarding secret
2. **Network policies**: Consider implementing Kubernetes Network Policies to restrict traffic
3. **Resource limits**: Set appropriate resource limits to prevent resource exhaustion
4. **Regular updates**: Keep your container images updated with security patches
5. **RBAC**: Implement proper Role-Based Access Control for cluster access

## Performance Tuning

### JVM Settings for Velocity

You can customize JVM arguments by modifying the Velocity Deployment:

```yaml
command: ["java"]
args:
  - "-Xms512M"
  - "-Xmx2G"
  - "-XX:+UseG1GC"
  - "-XX:G1HeapRegionSize=4M"
  - "-XX:+UnlockExperimentalVMOptions"
  - "-XX:+ParallelRefProcEnabled"
  - "-XX:+AlwaysPreTouch"
  - "-jar"
  - "./velocity.jar"
```

### Rust/MiniBit Optimization

Ensure your MiniBit binary is compiled with release optimizations:

```bash
cargo build --release
```

## Monitoring

Consider integrating with monitoring solutions:

- **Prometheus**: For metrics collection
- **Grafana**: For visualization
- **ELK Stack**: For log aggregation

Example ServiceMonitor for Prometheus:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: minibit-monitor
  namespace: minibit
spec:
  selector:
    matchLabels:
      app: minibit
  endpoints:
  - port: metrics
    interval: 30s
```

## Support

For issues and questions:
- Check the main [MiniBit repository](https://github.com/pythonsnake602/MiniBit)
- Review Kubernetes documentation for cluster-specific issues
- Consult Velocity documentation for proxy configuration

## License

This configuration follows the same license as the MiniBit project.

import type { RestartResponse } from "../../../../bindings/RestartResponse";
import type { ScaleResponse } from "../../../../bindings/ScaleResponse";
import type { DeployCanaryResponse } from "../../../../bindings/DeployCanaryResponse";
import type { PromoteCanaryResponse } from "../../../../bindings/PromoteCanaryResponse";

interface ActionTableProps {
    onError: (error: string | null) => void;
    isCanary?: boolean;
    image?: string;
}

export default function ActionTable({ onError, isCanary = false, image = "" }: ActionTableProps) {
    const handleRestart = async () => {
        try {
            onError(null);
            const res = await fetch('/api/restart', { method: 'POST' });
            if (!res.ok) throw new Error('API request failed');
            const data: RestartResponse = await res.json();
            if (!data.success) {
                onError('Restart failed: ' + (data.failure_msg || 'unknown error'));
            }
        } catch (err: any) {
            onError('Error restarting: ' + (err.message || err));
        }
    };

    const handleScale = async () => {
        const replicasInput = prompt('Enter new replica count:');
        if (replicasInput === null) return;
        const replica_count = parseInt(replicasInput, 10);
        if (isNaN(replica_count)) {
            onError('Please enter a valid number for replica count');
            return;
        }
        try {
            onError(null);
            const res = await fetch('/api/scale', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ replica_count }),
            });
            if (!res.ok) throw new Error('API request failed');
            const data: ScaleResponse = await res.json();
            if (!data.success) {
                onError('Scale failed: ' + (data.failure_msg || 'unknown error'));
            }
        } catch (err: any) {
            onError('Error scaling: ' + (err.message || err));
        }
    };

    const handleDeployCanary = async () => {
        const sha = prompt('Enter commit SHA to deploy:');
        if (!sha) return;
        try {
            onError(null);
            const res = await fetch('/api/deploy_canary', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sha }),
            });
            if (!res.ok) throw new Error('API request failed');
            const data: DeployCanaryResponse = await res.json();
            if (!data.success) {
                onError('Deploy Canary failed: ' + (data.failure_msg || 'unknown error'));
            }
        } catch (err: any) {
            onError('Error deploying canary: ' + (err.message || err));
        }
    };

    const handlePromoteCanary = async () => {
        try {
            onError(null);
            const res = await fetch('/api/promote_canary', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ sha: image }),
            });
            if (!res.ok) throw new Error('API request failed');
            const data: PromoteCanaryResponse = await res.json();
            if (!data.success) {
                onError('Promote Canary failed: ' + (data.failure_msg || 'unknown error'));
            }
        } catch (err: any) {
            onError('Error promoting canary: ' + (err.message || err));
        }
    };

    const btnClasses = "w-28 py-1 bg-slate-700 text-white rounded text-xs font-semibold hover:bg-slate-600 active:bg-slate-800 transition-colors shadow-sm text-center";

    if (isCanary) {
        return (
            <div className="flex gap-2 items-center justify-center p-2">
                <button onClick={handlePromoteCanary} className={btnClasses}>
                    Promote
                </button>
            </div>
        );
    }

    return (
        <div className="flex gap-2 items-center justify-center p-2">
            <button onClick={handleRestart} className={btnClasses}>
                Restart
            </button>
            <button onClick={handleScale} className={btnClasses}>
                Scale
            </button>
            <button onClick={handleDeployCanary} className={btnClasses}>
                Deploy Canary
            </button>
        </div>
    );
}
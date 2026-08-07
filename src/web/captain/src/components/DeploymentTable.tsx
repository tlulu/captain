import ActionTable from "./ActionTable";
import type { PodInfo } from "../../../../bindings/PodInfo";
import PodTable from "./PodTable";

interface DeploymentTableProps {
  pods: PodInfo[];
  onError: (error: string | null) => void;
  isCanary?: boolean;
}

export default function DeploymentTable({ pods, onError, isCanary = false }: DeploymentTableProps) {
  const image = pods[0]?.image || 'unknown';

  return (
    <div className="px-6 py-4 border border-slate-200 rounded-xl bg-white shadow-sm max-w-4xl w-full mx-auto text-center">
      <div className="flex flex-col sm:flex-row justify-between items-center gap-4 mb-4 border-b border-slate-100 pb-2">
        <h2 className="text-xs font-bold text-slate-500 uppercase tracking-wider">
          <span className="font-mono normal-case text-slate-800 text-[11px] bg-slate-100 px-1.5 py-0.5 rounded border border-slate-200">{image}</span>
        </h2>
        <ActionTable onError={onError} isCanary={isCanary} image={image} />
      </div>
      <PodTable pods={pods} />
    </div>
  );
}
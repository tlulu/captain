import type { PodInfo } from "../../../../bindings/PodInfo";

interface PodRowProps {
  pod: PodInfo;
}

export default function PodRow({ pod }: PodRowProps) {
  let themeClasses = "";
  let textMutedClasses = "";

  if (pod.status === 'Running') {
    themeClasses = "bg-emerald-50 border-emerald-200 text-emerald-950 hover:bg-emerald-100/50";
    textMutedClasses = "text-emerald-700";
  } else if (pod.status === 'Starting') {
    themeClasses = "bg-orange-50 border-orange-200 text-orange-950 hover:bg-orange-100/50";
    textMutedClasses = "text-orange-700";
  } else if (pod.status === 'Terminating') {
    themeClasses = "bg-rose-50 border-rose-200 text-rose-950 hover:bg-rose-100/50";
    textMutedClasses = "text-rose-700";
  } else {
    themeClasses = "bg-slate-50 border-slate-200 text-slate-950 hover:bg-slate-100/50";
    textMutedClasses = "text-slate-600";
  }

  return (
    <div className={`p-2 border rounded-md flex flex-col gap-0.5 shadow-sm relative text-left transition-colors duration-150 w-fit ${themeClasses}`}>
      <div className="flex justify-between items-center gap-2">
        <strong className="text-xs font-semibold break-all">{pod.name}</strong>
      </div>

      <div className={`text-[10px] font-mono flex flex-col ${textMutedClasses}`}>
        <div>IP: {pod.ip_address || 'None'}</div>
      </div>
    </div>
  );
}
import PodRow from "./PodRow";
import type { PodInfo } from "../../../../bindings/PodInfo";

interface PodTableProps {
    pods: PodInfo[];
}

export default function PodTable({ pods }: PodTableProps) {
    return (
        <div className="flex gap-4 py-2 px-0 overflow-x-auto">
            {pods.map((pod) => (
                <PodRow key={pod.name} pod={pod} />
            ))}
        </div>
    );
}
import type { ReactNode } from "react";

interface Props {
  title: string;
  actions?: ReactNode;
  children: ReactNode;
}

export function Panel({ title, actions, children }: Props) {
  return (
    <section className="panel">
      <header className="panel-header">
        <span className="text-accent">▚</span>
        <h2 className="flex-1">{title}</h2>
        {actions}
      </header>
      {children}
    </section>
  );
}

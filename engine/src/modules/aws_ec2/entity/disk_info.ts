import { Column, Entity, PrimaryGeneratedColumn, } from 'typeorm';

export enum DiskType {
  HDD = 'hdd',
  SSD = 'ssd',
}

@Entity()
export class DiskInfo {
  @PrimaryGeneratedColumn()
  id?: number;

  @Column({
    type: 'decimal',
  })
  sizeInGB: number;

  @Column({
    type: 'int',
  })
  count: number;

  @Column({
    type: 'enum',
    enum: DiskType,
  })
  diskType: DiskType;
}